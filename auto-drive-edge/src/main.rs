use gilrs::{Button, EventType, Gilrs};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Latest LeftTrigger2 value.
    //
    // We store f32 as its raw u32 bits because AtomicF32 doesn't
    // exist in std.
    let left_trigger2 = Arc::new(AtomicU32::new(0.0_f32.to_bits()));

    // Give the gamepad thread its own copy of the shared value.
    let trigger_for_thread = Arc::clone(&left_trigger2);

    // ------------------------------------------------------------
    // Gamepad thread
    // ------------------------------------------------------------
    //
    // This thread does nothing except wait for gamepad events.
    // next_event_blocking() puts the thread to sleep until an
    // event arrives.
    //
    thread::spawn(move || {
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
                    trigger_for_thread.store(value.to_bits(), Ordering::Relaxed);
                }

                EventType::ButtonPressed(Button::LeftTrigger2, _) => {
                    trigger_for_thread.store(1.0_f32.to_bits(), Ordering::Relaxed);
                }

                EventType::ButtonReleased(Button::LeftTrigger2, _) => {
                    trigger_for_thread.store(0.0_f32.to_bits(), Ordering::Relaxed);
                }

                _ => {}
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


    let mut previous_message = 1;
    loop {
        ticker.tick().await;

        // Read the latest value produced by the gamepad thread.
        let trigger = f32::from_bits(
            left_trigger2.load(Ordering::Relaxed)
        );

        // Convert 0.0..=1.0 into the i8 speed range.
        let mut speed = (trigger * i8::MAX as f32) as i8;
        if speed < 4 {
            speed = 0;
        }

        if (previous_message - speed).abs() <= 3 {
            continue;
        }
        previous_message = speed;


        let message =
            TheBeginnerCarIncomingMessages::SetMotorASpeed { speed };

        let bytes = postcard::to_stdvec(&message)?;
        let len = u32::try_from(bytes.len())?;

        // Send [4-byte length][postcard message]
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&bytes).await?;

        println!("LeftTrigger2: {trigger:.3} -> speed: {speed}");
    }
}