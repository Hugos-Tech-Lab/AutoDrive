use std::{sync::Arc, time::Duration};

use hardware_config::TheBeginnerCarMetadata;
use log::{info, warn};
use the_beginner_car_tcp_protocol::{
    MessageCodec, TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{broadcast, mpsc, watch},
    task::JoinHandle,
    time::{sleep, timeout},
};

#[derive(Debug)]
pub struct TheBeginnerCarConnectionHandle {
    pub connected_rx: watch::Receiver<bool>,
    outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
    pub incoming_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
    pub connection_handle: Arc<JoinHandle<()>>,
}

impl Clone for TheBeginnerCarConnectionHandle {
    fn clone(&self) -> Self {
        Self {
            connected_rx: self.connected_rx.clone(),
            outgoing_tx: self.outgoing_tx.clone(),
            incoming_rx: self.incoming_rx.resubscribe(),
            connection_handle: self.connection_handle.clone(),
        }
    }
}

impl TheBeginnerCarConnectionHandle {
    pub fn new(metadata: TheBeginnerCarMetadata) -> Self {
        let (outgoing_tx, internal_receiver) = mpsc::channel::<TheBeginnerCarIncomingMessages>(100);
        let (incoming_tx, incoming_rx) = broadcast::channel::<TheBeginnerCarOutgoingMessages>(100);
        let (connected_tx, connected_rx) = watch::channel(false);

        let connection_handle = tokio::spawn(Self::tcp_connection(
            metadata,
            internal_receiver,
            incoming_tx,
            connected_tx,
        ));

        Self {
            connected_rx,
            outgoing_tx,
            incoming_rx,
            connection_handle: Arc::new(connection_handle),
        }
    }

    pub fn is_connected(&self) -> bool {
        *self.connected_rx.borrow()
    }

    pub async fn send(&self, message: TheBeginnerCarIncomingMessages) -> bool {
        if !self.is_connected() {
            return false;
        }

        self.outgoing_tx.send(message).await.is_ok()
    }

    async fn tcp_connection(
        metadata: TheBeginnerCarMetadata,
        mut internal_receiver: mpsc::Receiver<TheBeginnerCarIncomingMessages>,
        broadcast_sender: broadcast::Sender<TheBeginnerCarOutgoingMessages>,
        connected_tx: watch::Sender<bool>,
    ) {
        const MAX_FRAME_LEN: usize = 16 * 1024;

        let addr = format!("{}:8080", metadata.wifi_mcu.ip);

        loop {
            if connected_tx.is_closed() {
                info!(
                    "All handles dropped for {}; stopping TCP task.",
                    metadata.number_plate
                );
                return;
            }

            let stream = match timeout(Duration::from_secs(5), TcpStream::connect(&addr)).await {
                Ok(Ok(stream)) => {
                    let _ = stream.set_nodelay(true);
                    let _ = connected_tx.send(true);
                    info!("Connected to {} at {addr}.", metadata.number_plate);
                    stream
                }
                Err(_) | Ok(Err(_)) => {
                    let _ = connected_tx.send(false);
                    warn!(
                        "Failed to connect to {} at {addr}. Trying again in {} seconds",
                        metadata.number_plate,
                        Duration::from_secs(5).as_secs()
                    );
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let (mut stream_read, mut stream_write) = stream.into_split();

            let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

            let writer_task = tokio::spawn({
                let number_plate = metadata.number_plate.clone();
                async move {
                    loop {
                        tokio::select! {
                            biased;
                            _ = shutdown_rx.changed() => {
                                break;
                            }
                            msg = internal_receiver.recv() => {
                                match msg {
                                    Some(message_to_car) => {
                                        let bytes = match message_to_car.to_bytes() {
                                            Ok(b) => b,
                                            Err(e) => {
                                                warn!("Failed to serialize outgoing message: {:?}", e);
                                                continue;
                                            }
                                        };
                                        if let Err(e) = stream_write.write_all(&bytes).await {
                                            warn!("Error writing to {number_plate}: {e}");
                                            break;
                                        }
                                    }
                                    None => {
                                        return None;
                                    }
                                }
                            }
                        }
                    }
                    Some(internal_receiver)
                }
            });

            loop {
                let read_result = timeout(Duration::from_secs(10), async {
                    let mut len_bytes = [0u8; 4];
                    stream_read.read_exact(&mut len_bytes).await?;

                     let len = u32::from_be_bytes(len_bytes) as usize;

                    if len > MAX_FRAME_LEN {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Payload length {} (raw bytes: {:02X?}) exceeds sanity limit. Framing error?",
                                len, len_bytes
                            ),
                        ));
                    }

                    let mut message_buf = vec![0u8; len];
                    stream_read.read_exact(&mut message_buf).await?;

                    Ok::<Vec<u8>, std::io::Error>(message_buf)
                })
                .await;

                match read_result {
                    Ok(Ok(message_buf)) => {
                        match postcard::from_bytes::<TheBeginnerCarOutgoingMessages>(&message_buf) {
                            Ok(message) => {
                                if broadcast_sender.send(message).is_err() {
                                    warn!(
                                        "No active incoming_rx subscribers; dropping incoming message."
                                    );
                                }
                            }
                            Err(err) => {
                                warn!(
                                    "Failed to deserialize message ({} bytes): {:?}",
                                    message_buf.len(),
                                    err
                                );
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        warn!("Error reading from {}: {e}", metadata.number_plate);
                        break;
                    }
                    Err(_) => {
                        warn!(
                            "Read timeout on {}: Car silently disconnected or lost power.",
                            metadata.number_plate
                        );
                        break;
                    }
                }
            }

            let _ = connected_tx.send(false);
            let _ = shutdown_tx.send(true);

            match writer_task.await {
                Ok(Some(rx)) => internal_receiver = rx,
                Ok(None) | Err(_) => return,
            }
        }
    }
}
