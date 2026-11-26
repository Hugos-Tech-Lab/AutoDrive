use std::sync::atomic::AtomicU32;
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub mod error;
pub mod info;

use anyhow::{Context, bail};
use async_io::{Async, Timer};
use flume::{Receiver, Sender};
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, FutureExt, try_join};
use log::error;
use log::info;
use std::io;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use the_beginner_car_spi_protocol::ResponseFromHardware;
use the_beginner_car_tcp_protocol::{
    TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};

use futures::executor::{LocalPool, LocalSpawner};
use futures::task::LocalSpawnExt;

#[derive(Debug, Clone)]
pub struct TcpServer {
    sender: Sender<TheBeginnerCarOutgoingMessages>,
    receiver: Receiver<TheBeginnerCarIncomingMessages>,
    thread_handle: Arc<JoinHandle<()>>,
}

impl TcpServer {
    pub fn new() -> Self {
        let (outgoing_messages_sender, outgoing_messages_receiver) = flume::unbounded();
        let (incoming_messages_sender, incoming_messages_receiver) = flume::unbounded();

        let thread_handle = std::thread::Builder::new()
            .name("tcp_handle".to_string())
            .stack_size(40 * 1000)
            .spawn({
                // let hardware_sender = hardware_sender.clone();
                || {
                    let mut local_executor = LocalPool::new();
                    let spawner = local_executor.spawner();
                    local_executor
                        .spawner()
                        .spawn_local(
                            async move {
                                accept(
                                    spawner,
                                    outgoing_messages_receiver,
                                    incoming_messages_sender,
                                )
                                .await
                                .unwrap();

                                Result::<_, anyhow::Error>::Ok(())
                            }
                            .map(Result::unwrap),
                        )
                        .unwrap();

                    local_executor.run();
                }
            })
            .unwrap();

        Self {
            sender: outgoing_messages_sender,
            receiver: incoming_messages_receiver,
            thread_handle: Arc::new(thread_handle),
        }
    }

    pub fn send(&self, message: TheBeginnerCarOutgoingMessages) {
        self.sender.send(message).unwrap();
    }

    pub fn receiver(&self) -> Receiver<TheBeginnerCarIncomingMessages> {
        self.receiver.clone()
    }
}

async fn handle(
    stream: Async<TcpStream>,
    outgoing_messages_receiver: Receiver<TheBeginnerCarOutgoingMessages>,
    incoming_messages_sender: Sender<TheBeginnerCarIncomingMessages>,
) {
    let (reader, writer) = stream.split();

    if let Err(err) = try_join!(
        handle_incoming(reader, incoming_messages_sender),
        handle_outgoing(writer, outgoing_messages_receiver),
    ) {
        error!("TCP connection ended: {err:#}");
    }
}

async fn handle_incoming<R>(
    mut reader: R,
    incoming_messages_sender: Sender<TheBeginnerCarIncomingMessages>,
) -> anyhow::Result<()>
where
    R: AsyncRead + Unpin,
{
    loop {
        let mut len_bytes = [0u8; 4];
        reader
            .read_exact(&mut len_bytes)
            .await
            .context("reading incoming frame length")?;

        let len = u32::from_be_bytes(len_bytes) as usize;

        if len > 16 * 1024 {
            bail!("Incoming message too large: {len} bytes");
        }

        let mut bytes = vec![0; len];
        reader
            .read_exact(&mut bytes)
            .await
            .context("reading incoming frame payload")?;

        let message: TheBeginnerCarIncomingMessages =
            postcard::from_bytes(&bytes).context("decoding incoming message")?;

        incoming_messages_sender.send_async(message).await;
    }
}

async fn handle_outgoing<W>(
    mut writer: W,
    outgoing_messages_receiver: Receiver<TheBeginnerCarOutgoingMessages>,
) -> anyhow::Result<()>
where
    W: AsyncWrite + Unpin,
{
    loop {
        let message = outgoing_messages_receiver
            .recv_async()
            .await
            .map_err(|_| anyhow::anyhow!("outgoing message channel closed"))?;

        let bytes = postcard::to_allocvec(&message).context("encoding outgoing message")?;

        let len =
            u32::try_from(bytes.len()).context("outgoing message exceeds u32 frame length")?;

        writer
            .write_all(&len.to_be_bytes())
            .await
            .context("writing outgoing frame length")?;

        writer
            .write_all(&bytes)
            .await
            .context("writing outgoing frame payload")?;
    }
}

async fn wait_for_hardware_response(
    receiver: mpsc::Receiver<Result<ResponseFromHardware, anyhow::Error>>,
    timeout: Duration,
) -> anyhow::Result<ResponseFromHardware> {
    let deadline = Instant::now() + timeout;

    loop {
        match receiver.try_recv() {
            Ok(Ok(response)) => return Ok(response),
            Ok(Err(err)) => return Err(err).context("hardware request failed"),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                bail!("hardware response channel disconnected");
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
        }

        if Instant::now() >= deadline {
            bail!("timed out waiting for hardware response");
        }

        Timer::after(Duration::from_millis(10)).await;
    }
}

pub async fn accept(
    spawner: LocalSpawner,
    outgoing_messages_receiver: Receiver<TheBeginnerCarOutgoingMessages>,
    incoming_messages_sender: Sender<TheBeginnerCarIncomingMessages>,
) -> Result<(), io::Error> {
    info!("Binding to port 8080");

    let addr = "0.0.0.0:8080".to_socket_addrs()?.next().unwrap();
    let listener = Async::<TcpListener>::bind(addr)?;
    let edge_connection_len = AtomicU32::new(0);

    // if edge_connection_len.load(Ordering::Relaxed) > 0 {
    //     bail!("too many edge connections");
    // }

    // edge_connection_len.fetch_add(1, Ordering::Relaxed);
    // // let mut bytes = [0u8; 1];
    // // reader
    // //     .read_exact(&mut bytes)
    // //     .await
    // //     .context("reading incoming frame length")?;

    // // let authentication_id = u8::from_be_bytes(bytes);
    // // if authentication_id == 3 {

    // // }

    loop {
        let stream = listener.accept().await;
        let outgoing_messages_receiver = outgoing_messages_receiver.clone();
        let incoming_messages_sender = incoming_messages_sender.clone();
        match stream {
            Ok((stream, addr)) => {
                info!("Accepted client {addr}");

                spawner
                    .spawn_local(handle(
                        stream,
                        outgoing_messages_receiver,
                        incoming_messages_sender,
                    ))
                    .unwrap();
            }
            Err(e) => {
                error!("Error: {e}");
            }
        }
    }
}
