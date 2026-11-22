use std::{thread, time::Duration};

use hardware_config::TheBeginnerCarMetadata;
use log::{info, warn};
use the_beginner_car_tcp_protocol::{
    MessageCodec, TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    select,
    sync::{broadcast, mpsc, watch},
    task::JoinHandle,
    time::timeout,
};

#[derive(Debug)]
pub struct TheBeginnerCarComms {
    pub metadata: TheBeginnerCarMetadata,
    pub connected_rx: watch::Receiver<bool>,
    outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
    pub incoming_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
}

impl Clone for TheBeginnerCarComms {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata.clone(),
            connected_rx: self.connected_rx.clone(),
            outgoing_tx: self.outgoing_tx.clone(),
            incoming_rx: self.incoming_rx.resubscribe(),
        }
    }
}

impl TheBeginnerCarComms {
    pub fn new(
        device_metadata: TheBeginnerCarMetadata,
        // vehicle_telemetry: TheBeginnerCarVehicleHardwareTelemetry,
    ) -> (Self, JoinHandle<()>) {
        let (outgoing_tx, internal_receiver) = mpsc::channel::<TheBeginnerCarIncomingMessages>(100);
        let (incoming_tx, incoming_rx) = broadcast::channel::<TheBeginnerCarOutgoingMessages>(100);

        let (connected_tx, connected_rx) = watch::channel(false);

        let connection_handle = tokio::spawn(Self::tcp_connection(
            device_metadata.clone(),
            internal_receiver,
            incoming_tx.clone(),
            connected_tx.clone(),
        ));

        (
            Self {
                connected_rx,
                outgoing_tx,
                incoming_rx,
                metadata: device_metadata,
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

        if let Err(_) = self.outgoing_tx.send(message).await {
            return false;
        }
        true
    }

    async fn tcp_connection(
        metadata: TheBeginnerCarMetadata,
        mut internal_receiver: mpsc::Receiver<TheBeginnerCarIncomingMessages>,
        broadcast_sender: broadcast::Sender<TheBeginnerCarOutgoingMessages>,
        connected_tx: watch::Sender<bool>,
    ) {
        loop {
            let ip = metadata.wifi_mcu.ip;
            let ip = format!("{ip}:8080");

            let stream = match timeout(Duration::from_secs(5), TcpStream::connect(ip.clone())).await
            {
                Ok(Ok(stream)) => {
                    connected_tx.send(true).unwrap();
                    info!("Connected to {} at {ip}.", metadata.number_plate);
                    stream
                }
                Err(_) | Ok(Err(_)) => {
                    connected_tx.send(false).unwrap();
                    let ip = ip.clone();
                    eprintln!(
                        "Failed to connect to {} at {ip}. Trying again in 5 seconds",
                        metadata.number_plate
                    );
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
            };

            let (mut stream_read, mut stream_write) = stream.into_split();
            loop {
                select! {
                    // car must send message every 10 seconds
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
                            Ok(Ok(message_buf)) => {
                                let message: TheBeginnerCarOutgoingMessages = match postcard::from_bytes(&message_buf) {
                                    Ok(message) => message,
                                    Err(err) => {
                                        warn!("Failed to deserialize message: {:?}", err);
                                        continue;
                                    }
                                };
                                if let Err(_) = broadcast_sender.send(message) {
                                    warn!("Did not send broadcast message, assuming all receivers dropped, that's strange?")
                                };
                            }
                            Ok(Err(e)) => {
                                warn!("Error reading from car: {e}");
                                connected_tx.send(false).unwrap();
                                break;
                            }
                            Err(_) => {
                                warn!("Read timeout: Car silently disconnected or lost power.");
                                connected_tx.send(false).unwrap();
                                break;
                            }
                        }
                    }
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
