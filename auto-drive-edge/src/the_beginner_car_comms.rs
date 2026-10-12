use std::{thread, time::Duration};

use opentelemetry::KeyValue;
use the_beginner_car_tcp_protocol::{
    MessageCodec, TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt}, net::{TcpStream, lookup_host}, select, sync::{broadcast, mpsc}, task::JoinHandle, time::timeout,
};

use crate::{
    devices_config::{DeviceBuild, DeviceMetadata},
    // the_beginner_cars::TheBeginnerCarVehicleHardwareTelemetry,
};

//
// pub struct VehicleHardwareTelemetry {
//     // motor_a_set_speed: Gauge<f64>,
//     // motor_b_set_speed: Gauge<f64>,
//     motor_battery_voltage: Gauge<f64>,
//     // wifi_battery_voltage: Gauge<f64>,
// }

// impl VehicleHardwareTelemetry {
//     pub fn new(meter: Meter, identifier: [KeyValue; 1]) -> Self {

//         motor_battery_voltage.record(2.0, &identifier);

//         Self {
//             motor_battery_voltage,
//         }
//     }
// }

// Answers questions like:
// Is CPU usage too high?
// What's the average over 5 minutes?
// Hows the Memory?
// pub struct SoftwareTelemetry {
//     cpu_percentage: Gauge,
// }

// impl Default for SoftwareTelemetry {
//     fn default() -> Self {
//         Self {
//             cpu_percentage: gauge!("cpu_percentage"),
//         }
//     }
// }

// impl SoftwareTelemetry {
//     pub fn new() -> Self {
//         Self {
//             cpu_percentage: gauge!("cpu_percentage"),
//         }
//     }
// }

// pub struct TheBeginnerCarHardwareState {
//     //
// }

pub struct TheBeginnerCarComms {
    pub build: DeviceBuild,
    mdns_address: String,
    // connected: bool,
    pub outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
    pub incoming_tx: broadcast::Sender<TheBeginnerCarOutgoingMessages>,
    connection_handle: JoinHandle<()>
}

impl TheBeginnerCarComms {
    pub fn new(
        device_metadata: DeviceMetadata,
        // vehicle_telemetry: TheBeginnerCarVehicleHardwareTelemetry,
    ) -> Self {
        let (outgoing_tx, internal_receiver) = mpsc::channel::<TheBeginnerCarIncomingMessages>(100);
        let (incoming_tx, _) = broadcast::channel::<TheBeginnerCarOutgoingMessages>(100);

        let connection_handle = tokio::spawn({
            let mdns_address = device_metadata.mdns_address.clone();
            let incoming_tx = incoming_tx.clone();
            async {
                Self::long_lived_connection(mdns_address, internal_receiver, incoming_tx).await;
            }
        });

        Self {
            build: device_metadata.build,
            mdns_address: device_metadata.mdns_address,
            // connected: false,
            outgoing_tx,
            incoming_tx,
            connection_handle
        }
    }

    pub fn connected() -> bool {
        true // TODO this should be a atomic
    }

    async fn long_lived_connection(
        mdns_address: String,
        mut receiver: mpsc::Receiver<TheBeginnerCarIncomingMessages>,
        broadcast_sender: broadcast::Sender<TheBeginnerCarOutgoingMessages>,
    ) {
        loop {
            let m_dns = mdns_address.clone();

            let host = format!("{m_dns}:8080");
            let Ok(mut addresses) = lookup_host(&host).await else {
                println!("Could not find {host}. Trying again in 10 seconds");
                thread::sleep(Duration::from_secs(10));
                continue;
            };

            let addr = addresses
                .next()
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "No socket addresses found")
                })
                .unwrap();

            let stream = match timeout(Duration::from_secs(5), TcpStream::connect(addr.clone()))
                .await
            {
                Ok(Ok(stream)) => {
                    println!("Connected to {host}.");
                    stream
                }

                Ok(Err(error)) => {
                    let addr = addr.clone();
                    eprintln!(
                        "Failed to connect to {addr} error: {error}. Trying again in 10 seconds"
                    );
                    thread::sleep(Duration::from_secs(10));
                    continue;
                }

                Err(_) => {
                    eprintln!("Connection timed out after 5 seconds. Trying again in 10 seconds");
                    thread::sleep(Duration::from_secs(10));
                    continue;
                }
            };

            let (mut stream_read, mut stream_write) = stream.into_split();
            loop {
                select! {
                    // Incoming data from the car
                    result = async {
                        let mut len_bytes = [0u8; 4];
                        stream_read.read_exact(&mut len_bytes).await?;

                        let len = u32::from_be_bytes(len_bytes) as usize;
                        let mut message_buf = vec![0u8; len];

                        stream_read.read_exact(&mut message_buf).await?;

                        Ok::<Vec<u8>, std::io::Error>(message_buf)
                    } => {
                        match result {
                            Ok(message_buf) => {
                                let message: TheBeginnerCarOutgoingMessages =
                                    postcard::from_bytes(&message_buf).unwrap();

                                // Forward incoming message to broadcast subscribers
                                let _ = broadcast_sender.send(message);
                            }
                            Err(e) => {
                                eprintln!("Error reading from car: {e}");
                                break;
                            }
                        }
                    }

                    // Outgoing data to the car
                    message = receiver.recv() => {
                        match message {
                            Some(message_to_car) => {
                                let bytes = message_to_car.to_bytes().unwrap();

                                if let Err(e) = stream_write.write_all(&bytes).await {
                                    eprintln!("Error writing to car: {e}");
                                    break;
                                }
                            }
                            None => {
                                // All senders dropped
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
