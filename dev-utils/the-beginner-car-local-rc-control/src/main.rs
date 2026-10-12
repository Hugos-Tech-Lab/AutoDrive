use the_beginner_car_tcp_protocol::TheBeginnerCarIncomingMessages;
use std::{
    error::Error,
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{interval, timeout},
};


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Connecting...");

    // let auto_drive_wasm = include_bytes!("../auto-drive.wasm");
    let auto_drive_wasm = include_bytes!("../stop.wasm");

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

    let message = TheBeginnerCarIncomingMessages::RunAutoScript { data: auto_drive_wasm.to_vec() };


    let bytes = postcard::to_stdvec(&message).unwrap();
    let len = u32::try_from(bytes.len()).unwrap();

    // Send [4-byte length][postcard message]
    stream.write_all(&len.to_be_bytes()).await.unwrap();
    stream.write_all(&bytes).await.unwrap();
    Ok(())
}
