use std::sync::atomic::AtomicU32;
use std::sync::mpsc::{self, SyncSender};
use std::time::{Duration, Instant};

pub mod auto_script;
pub mod error;
pub mod info;
pub mod set_motor_a_speed;
pub mod set_motor_b_speed;
pub mod update;
pub mod verify_and_set_valid;

use anyhow::{Context, bail};
use async_io::{Async, Timer};
use flume::Receiver;
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, try_join};
use log::error;
use log::info;
use std::io;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use the_beginner_car_spi_protocol::{RequestToHardware, ResponseFromHardware};
use the_beginner_car_tcp_protocol::{
    TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};

use futures::executor::LocalSpawner;
use futures::task::LocalSpawnExt;

use crate::HardwareMessage;

async fn handle(
    stream: Async<TcpStream>,
    hardware_sender: SyncSender<HardwareMessage>,
    receiver_tx: Receiver<TheBeginnerCarOutgoingMessages>,
) {
    let (reader, writer) = stream.split();

    if let Err(err) = try_join!(
        handle_incoming(reader, hardware_sender),
        handle_outgoing(writer, receiver_tx),
    ) {
        error!("TCP connection ended: {err:#}");
    }
}

async fn handle_incoming<R>(
    mut reader: R,
    hardware_sender: SyncSender<HardwareMessage>,
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

        match message {
            TheBeginnerCarIncomingMessages::SetMotorASpeed { speed } => {
                set_motor_a_speed::set_motor_a_speed_on_hardware(hardware_sender.clone(), speed)
                    .map_err(|err| anyhow::anyhow!("{err:?}"))?;
            }

            TheBeginnerCarIncomingMessages::SetMotorBSpeed { speed } => {
                set_motor_b_speed::set_motor_b_speed_on_hardware(hardware_sender.clone(), speed)
                    .map_err(|err| anyhow::anyhow!("{err:?}"))?;
            }

            TheBeginnerCarIncomingMessages::InformPositionFromCamera { x, y } => {
                println!("InformPositionFromCamera: {:?} {:?}", x, y);
            }

            TheBeginnerCarIncomingMessages::RunAutoScript { script } => {
                info!("run auto script");

                let (sender, receiver) = std::sync::mpsc::sync_channel(1);

                hardware_sender
                    .send(HardwareMessage {
                        request: RequestToHardware::RunAutoScript {
                            data: script.script,
                        },
                        response_tx: sender,
                    })
                    .context("sending RunAutoScript to hardware")?;

                let response = wait_for_hardware_response(receiver, Duration::from_secs(5)).await?;
                match response {
                    ResponseFromHardware::AutoScriptStarted { id } => {
                        dbg!(id);
                    }
                    _ => {}
                }
            }

            TheBeginnerCarIncomingMessages::CancelAutoScript { run_uuid } => {
                let (sender, receiver) = std::sync::mpsc::sync_channel(1);
                hardware_sender
                    .send(HardwareMessage {
                        request: RequestToHardware::StopAutoScript,
                        response_tx: sender,
                    })
                    .context("sending StopAutoScript to hardware")?;

                let response = wait_for_hardware_response(receiver, Duration::from_secs(5)).await?;
                match response {
                    ResponseFromHardware::Ok => {
                        dbg!("ok");
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn handle_outgoing<W>(
    mut writer: W,
    receiver: Receiver<TheBeginnerCarOutgoingMessages>,
) -> anyhow::Result<()>
where
    W: AsyncWrite + Unpin,
{
    loop {
        let message = receiver
            .recv_async()
            .await
            .map_err(|_| anyhow::anyhow!("outgoing message channel closed"))?;

        if message == TheBeginnerCarOutgoingMessages::AutoScriptStopped {
            dbg!("stopped");
        }

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
    hardware_sender: SyncSender<HardwareMessage>,
    receiver_tx: Receiver<TheBeginnerCarOutgoingMessages>,
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
        let receiver_tx = receiver_tx.clone();
        match stream {
            Ok((stream, addr)) => {
                info!("Accepted client {addr}");

                spawner
                    .spawn_local(handle(stream, hardware_sender.clone(), receiver_tx))
                    .unwrap();
            }
            Err(e) => {
                error!("Error: {e}");
            }
        }
    }
}
