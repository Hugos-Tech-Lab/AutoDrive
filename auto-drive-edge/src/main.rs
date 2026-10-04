use std::{error::Error, thread::sleep, time::Duration};
use gilrs::{Gilrs, Button, Event};

use serde::{Deserialize, Serialize};
use tokio::{
    io::AsyncWriteExt, net::TcpStream, time::timeout,
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
    let mut stream = match timeout(
        Duration::from_secs(5),
        TcpStream::connect("192.168.0.180:8080"),
    )
    .await
    {
        Ok(result) => {
            match result {
                Ok(stream) => {
                    println!("Connected!");
                    stream
                }
                Err(error) => {
                    eprintln!("Failed to connect: {error}");
                    return Ok(());
                }
            }
        }
        Err(_) => {
            eprintln!("Connection timed out after 5 seconds");
            return Ok(());
        }
    };

    let message = TheBeginnerCarIncomingMessages::SetMotorBSpeed { speed: 0 };

    let bytes = postcard::to_stdvec(&message)?;
    let len = u32::try_from(bytes.len())?;

    let mut gilrs = Gilrs::new().unwrap();


    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }


    let mut active_gamepad = None;

    loop {
        println!("hi");
        // Examine new events
        while let Some(Event { id, event, time, .. }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
            active_gamepad = Some(id);
        }

        // You can also use cached gamepad state
        if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            if gamepad.is_pressed(Button::South) {
                println!("Button South is pressed (XBox - A, PS - X)");
            }
        }

         sleep(Duration::from_secs(1));
    }

    loop {
    // Send [4-byte length][postcard message]
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&bytes).await?;

        println!("Sent SetMotorASpeed {{ speed: 42 }}");
        sleep(Duration::from_secs(1));
    }


    Ok(())
}
