use gilrs::{Axis, EventType, Gilrs};
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

/// Convert joystick value (-1.0..=1.0) to motor speed.
///
/// -1.0 = full reverse
///  0.0 = stopped
/// +1.0 = full forward
fn joystick_to_speed(value: f32) -> i8 {
    // Deadzone to prevent joystick drift.
    let deadzone = 0.10;

    if value.abs() < deadzone {
        return 0;
    }

    // Remap the value outside the deadzone so that
    // movement still reaches full speed.
    let value = if value > 0.0 {
        (value - deadzone) / (1.0 - deadzone)
    } else {
        (value + deadzone) / (1.0 - deadzone)
    };

    (value.clamp(-1.0, 1.0) * 127.0) as i8
}

async fn send_motor_speed(
    previous_value: &mut i8,
    speed: i8,
    stream: &mut TcpStream,
    is_left: bool,
) {
    // Don't send tiny changes caused by joystick noise.
    if ((*previous_value as i16) - (speed as i16)).abs() <= 3 {
        return;
    }

    *previous_value = speed;

    let message = if is_left {
        TheBeginnerCarIncomingMessages::SetMotorBSpeed { speed }
    } else {
        TheBeginnerCarIncomingMessages::SetMotorASpeed { speed }
    };

    let bytes = postcard::to_stdvec(&message).unwrap();
    let len = u32::try_from(bytes.len()).unwrap();

    // Send [4-byte length][postcard message]
    stream.write_all(&len.to_be_bytes()).await.unwrap();
    stream.write_all(&bytes).await.unwrap();

    println!(
        "{} motor: {}",
        if is_left { "Left" } else { "Right" },
        speed
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // ------------------------------------------------------------
    // Shared joystick state
    // ------------------------------------------------------------
    //
    // X = steering
    // Y = forward/backward
    //
    // AtomicU32 is used to store f32 bits.
    //

    let joystick_x = Arc::new(AtomicU32::new(0.0_f32.to_bits()));
    let joystick_y = Arc::new(AtomicU32::new(0.0_f32.to_bits()));

    // ------------------------------------------------------------
    // Gamepad thread
    // ------------------------------------------------------------

    thread::spawn({
        let joystick_x = joystick_x.clone();
        let joystick_y = joystick_y.clone();

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
                    // Left stick X
                    EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                        joystick_x.store(value.to_bits(), Ordering::Relaxed);
                    }

                    // Left stick Y
                    EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                        joystick_y.store(value.to_bits(), Ordering::Relaxed);
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
    // Send motor speeds
    // ------------------------------------------------------------

    let mut ticker = interval(Duration::from_millis(20));

    let mut previous_left_motor: i8 = 0;
    let mut previous_right_motor: i8 = 0;

    loop {
        ticker.tick().await;

        // Read joystick values.
        let x = f32::from_bits(joystick_x.load(Ordering::Relaxed));
        let y = f32::from_bits(joystick_y.load(Ordering::Relaxed));

        // gilrs normally reports LeftStickY as:
        //
        //   -1 = up
        //   +1 = down
        //
        // We want:
        //
        //   +1 = forward
        //   -1 = reverse
        //
        let forward = -y;
        let steering = -x;

        // Differential drive mixing.
        //
        // Forward + steering:
        //
        //             left     right
        // straight    +1       +1
        // turn left   -/+      +1
        // turn right  +1       -/+
        //
        let left = forward + steering;
        let right = forward - steering;

        // Normalize so that neither motor exceeds [-1, 1].
        let max = left.abs().max(right.abs()).max(1.0);

        let left = left / max;
        let right = right / max;

        let left_speed = joystick_to_speed(left);
        let right_speed = joystick_to_speed(right);

        send_motor_speed(
            &mut previous_left_motor,
            left_speed,
            &mut stream,
            true,
        )
        .await;

        send_motor_speed(
            &mut previous_right_motor,
            right_speed,
            &mut stream,
            false,
        )
        .await;
    }
}
