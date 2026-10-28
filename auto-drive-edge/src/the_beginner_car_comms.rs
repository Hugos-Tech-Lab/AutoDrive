use std::{thread, time::Duration};

use log::{info, warn};
use the_beginner_car_tcp_protocol::{
    MessageCodec, TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, lookup_host},
    select,
    sync::{broadcast, mpsc, watch},
    task::JoinHandle,
    time::timeout,
};

use crate::devices_config::HardwareMetadata;

#[derive(Clone, Debug)]
pub struct TheBeginnerCarComms {
    pub number_plate: String,
    pub mdns_address: String,
    pub connected_rx: watch::Receiver<bool>,
    outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
    pub incoming_tx: broadcast::Sender<TheBeginnerCarOutgoingMessages>,
}

impl TheBeginnerCarComms {
    pub fn new(
        device_metadata: HardwareMetadata,
        // vehicle_telemetry: TheBeginnerCarVehicleHardwareTelemetry,
    ) -> (Self, JoinHandle<()>) {
        let (outgoing_tx, internal_receiver) = mpsc::channel::<TheBeginnerCarIncomingMessages>(100);
        let (incoming_tx, _) = broadcast::channel::<TheBeginnerCarOutgoingMessages>(100);

        let (connected_tx, connected_rx) = watch::channel(false);

        let connection_handle = tokio::spawn({
            let mdns_address = device_metadata.mdns_address.clone();
            let incoming_tx = incoming_tx.clone();
            let connected_tx = connected_tx.clone();
            async {
                Self::long_lived_connection(
                    mdns_address,
                    internal_receiver,
                    incoming_tx,
                    connected_tx,
                )
                .await;
            }
        });

        (
            Self {
                number_plate: device_metadata.number_plate,
                mdns_address: device_metadata.mdns_address,
                connected_rx,
                outgoing_tx,
                incoming_tx,
            },
            connection_handle,
        )
    }

    pub fn is_connected(&self) -> bool {
        self.connected_rx.borrow().clone()
    }

    pub async fn send(&self, message: TheBeginnerCarIncomingMessages) -> bool {
        if !self.is_connected() {
            return false;
        }

        self.outgoing_tx.send(message).await.unwrap();
        true
    }

    async fn long_lived_connection(
        mdns_address: String,
        mut internal_receiver: mpsc::Receiver<TheBeginnerCarIncomingMessages>,
        broadcast_sender: broadcast::Sender<TheBeginnerCarOutgoingMessages>,
        connected_tx: watch::Sender<bool>,
    ) {
        loop {
            let m_dns = mdns_address.clone();

            let host = format!("{m_dns}:8080");
            let Ok(mut addresses) = lookup_host(&host).await else {
                connected_tx.send(false).unwrap();
                warn!("Could not find {host}. Trying again in 10 seconds");
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
                    connected_tx.send(true).unwrap();
                    info!("Connected to {host}.");
                    stream
                }

                Ok(Err(error)) => {
                    connected_tx.send(false).unwrap();
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
                    // we assume the car sends a message at least once every 10 seconds...
                    result = timeout(Duration::from_secs(10), async {
                        let mut len_bytes = [0u8; 4];
                        stream_read.read_exact(&mut len_bytes).await?;

                        let len = u32::from_be_bytes(len_bytes) as usize;

                        if len > 16 * 1024 {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Payload length {} exceeds sanity limit. Framing error?", len)
                            ));
                        }

                        let mut message_buf = vec![0u8; len];
                        stream_read.read_exact(&mut message_buf).await?;

                        Ok::<Vec<u8>, std::io::Error>(message_buf)
                    }) => {
                        match result {
                            // Read succeeded
                            Ok(Ok(message_buf)) => {
                                let message: TheBeginnerCarOutgoingMessages = match postcard::from_bytes(&message_buf) {
                                    Ok(message) => message,
                                    Err(err) => {
                                        eprintln!("Failed to deserialize message: {:?}", err);
                                        continue;
                                    }
                                };

                                if let Err(_) = broadcast_sender.send(message) {
                                    warn!("Did not send broadcast message, assuming all receivers dropped, that's strange?")
                                };
                            }
                            Ok(Err(e)) => {
                                eprintln!("Error reading from car: {e}");
                                connected_tx.send(false).unwrap();
                                break;
                            }
                            Err(_) => {
                                eprintln!("Read timeout: Car silently disconnected or lost power.");
                                connected_tx.send(false).unwrap();
                                break;
                            }
                        }
                    }

                    // Outgoing data to the car
                    message = internal_receiver.recv() => {
                        match message {
                            Some(message_to_car) => {
                                let bytes = message_to_car.to_bytes().unwrap();

                                if let Err(e) = stream_write.write_all(&bytes).await {
                                    eprintln!("Error writing to car: {e}");
                                    connected_tx.send(false).unwrap();
                                    break;
                                } else {
                                    connected_tx.send(true).unwrap();
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
