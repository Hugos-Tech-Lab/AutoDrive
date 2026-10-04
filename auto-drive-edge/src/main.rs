use gilrs::{Button, EventType, Gilrs};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    thread,
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{interval, timeout},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum TheBeginnerCarIncomingMessages {
    SetMotorASpeed { speed: i8 },
    SetMotorBSpeed { speed: i8 },
    InformPositionFromCamera { x: i32, y: i32 },
    InstallWasm { bytes: Vec<u8> },
    RunWasm,
}

pub async fn maybe_send_message(previous_value: &mut i8, latest_value: &AtomicU32, stream: &mut TcpStream, is_left: bool) {
    // Read the latest value produced by the gamepad thread.
    let left = f32::from_bits(latest_value.load(Ordering::Relaxed));

    // Convert 0.0..=1.0 into the i8 speed range.
    let mut speed = (left * i8::MAX as f32) as i8;
    if speed < 4 {
        speed = 0;
    }

    if (*previous_value - speed).abs() <= 3 {
        return;
    }
    *previous_value = speed;

    let message;
    if is_left {
        message = TheBeginnerCarIncomingMessages::SetMotorASpeed { speed };
    } else {
        message = TheBeginnerCarIncomingMessages::SetMotorBSpeed { speed };
    }

    let bytes = postcard::to_stdvec(&message).unwrap();
    let len = u32::try_from(bytes.len()).unwrap();

    // Send [4-byte length][postcard message]
    stream.write_all(&len.to_be_bytes()).await.unwrap();
    stream.write_all(&bytes).await.unwrap();

    println!("SetMotorASpeed: speed: {speed}");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Latest LeftTrigger2 value.
    //
    // We store f32 as its raw u32 bits because AtomicF32 doesn't
    // exist in std.
    let left_trigger2 = Arc::new(AtomicU32::new(0.0_f32.to_bits()));
    let right_trigger2 = Arc::new(AtomicU32::new(0.0_f32.to_bits()));

    // ------------------------------------------------------------
    // Gamepad thread
    // ------------------------------------------------------------
    //
    // This thread does nothing except wait for gamepad events.
    // next_event_blocking() puts the thread to sleep until an
    // event arrives.
    //
    thread::spawn({
        // Give the gamepad thread its own copy of the shared value.
        let left_trigger2 = left_trigger2.clone();
        let right_trigger2 = right_trigger2.clone();
        move || {
            let mut gilrs = match Gilrs::new() {
                Ok(gilrs) => gilrs,
                Err(error) => {
                    eprintln!("Failed to initialize gilrs: {error}");
                    return;
                }
            };

            println!("Gamepad thread started.");

            loop {
                let Some(event) = gilrs.next_event_blocking(None) else {
                    continue;
                };

                match event.event {
                    EventType::ButtonChanged(Button::LeftTrigger2, value, _) => {
                        left_trigger2.store(value.to_bits(), Ordering::Relaxed);
                    }

                    EventType::ButtonPressed(Button::LeftTrigger2, _) => {
                        left_trigger2.store(1.0_f32.to_bits(), Ordering::Relaxed);
                    }

                    EventType::ButtonReleased(Button::LeftTrigger2, _) => {
                        left_trigger2.store(0.0_f32.to_bits(), Ordering::Relaxed);
                    }
                    EventType::ButtonChanged(Button::RightTrigger2, value, _) => {
                        right_trigger2.store(value.to_bits(), Ordering::Relaxed);
                    }

                    EventType::ButtonPressed(Button::RightTrigger2, _) => {
                        right_trigger2.store(1.0_f32.to_bits(), Ordering::Relaxed);
                    }

                    EventType::ButtonReleased(Button::RightTrigger2, _) => {
                        right_trigger2.store(0.0_f32.to_bits(), Ordering::Relaxed);
                    }

                    _ => {}
                }
            }
        }
    });

    // ------------------------------------------------------------
    // TCP connection
    // ------------------------------------------------------------

    println!("Connecting...");

    let mut stream = match timeout(
        Duration::from_secs(5),
        TcpStream::connect("192.168.0.180:8080"),
    )
    .await
    {
        Ok(Ok(stream)) => {
            println!("Connected!");
            stream
        }

        Ok(Err(error)) => {
            eprintln!("Failed to connect: {error}");
            return Ok(());
        }

        Err(_) => {
            eprintln!("Connection timed out after 5 seconds");
            return Ok(());
        }
    };

    // ------------------------------------------------------------
    // Send the current trigger value every 50 ms
    // ------------------------------------------------------------

    let mut ticker = interval(Duration::from_millis(20));

    let mut previous_right_message = 1;
    let mut previous_left_message = 1;
    loop {
        ticker.tick().await;

        maybe_send_message(&mut previous_left_message, &left_trigger2, &mut stream, true).await;
        maybe_send_message(&mut previous_right_message, &right_trigger2, &mut stream, false).await;

        // println!("LeftTrigger2: {trigger:.3} -> speed: {speed}");
    }
}
