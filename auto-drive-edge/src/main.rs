use std::{
    error::Error,
    sync::{Arc, atomic::AtomicU32},
    time::Duration,
};
use tokio::{net::TcpStream, time::timeout};

use crate::{auto_drive_edge_server::AutoDriveEdgeServer, devices_config::DeviceConfig};

pub mod auto_drive_edge_server;
pub mod devices_config;
pub mod the_beginner_car;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let contents = include_str!("../device_config.toml");
    let device_config: DeviceConfig = toml::from_str(&contents)?;
    let auto_drive_edge_server = AutoDriveEdgeServer::new(device_config);

    // ----------------------------------------------------------
    // TCP connection
    // ------------------------------------------------------------

    println!("Connecting...");



    // ------------------------------------------------------------
    // Send the current trigger value every 50 ms
    // ------------------------------------------------------------

    // let mut ticker = interval(Duration::from_millis(20));

    // let mut previous_right_message = 1;
    // let mut previous_left_message = 1;
    // loop {
    //     ticker.tick().await;

    // }

    Ok(())
}
