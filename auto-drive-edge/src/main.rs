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

pub mod the_beginner_car;
pub mod auto_drive_edge_server;


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
    // READ LIST OF DEVICES
    // 

    // Latest LeftTrigger2 value.
    //
    // We store f32 as its raw u32 bits because AtomicF32 doesn't
    // exist in std.
    let left_trigger2 = Arc::new(AtomicU32::new(0.0_f32.to_bits()));
    let right_trigger2 = Arc::new(AtomicU32::new(0.0_f32.to_bits()));


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

        // maybe_send_message(
        //     &mut previous_left_message,
        //     &left_trigger2,
        //     &mut stream,
        //     true,
        // )
        // .await;
        // maybe_send_message(
        //     &mut previous_right_message,
        //     &right_trigger2,
        //     &mut stream,
        //     false,
        // )
        // .await;

        // println!("LeftTrigger2: {trigger:.3} -> speed: {speed}");
    }
}
