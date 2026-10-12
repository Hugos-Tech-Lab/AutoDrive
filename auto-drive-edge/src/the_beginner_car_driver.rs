use std::{sync::Arc, time::Duration};

use the_beginner_car_tcp_protocol::{
    TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    sync::{
        Mutex, broadcast,
        mpsc::{self},
    },
    task::JoinHandle,
    time::{timeout, timeout_at},
};

pub struct HardwareState {
    queued_scripts: Vec<Vec<u8>>,
}

pub struct TheBeginnerCarDriver {
    state: Arc<Mutex<HardwareState>>,
    pub outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
    pub incoming_rx: broadcast::Sender<TheBeginnerCarOutgoingMessages>,
    connection_handle: JoinHandle<()>,
}

impl TheBeginnerCarDriver {
    pub fn new(
        outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
        incoming_rx: broadcast::Sender<TheBeginnerCarOutgoingMessages>,
    ) -> Self {
        let state = Arc::new(Mutex::new(HardwareState {
            queued_scripts: vec![],
        }));

        let connection_handle = tokio::spawn({
            // let mdns_address = device_metadata.mdns_address.clone();
            // let incoming_tx = incoming_tx.clone();
            let state = state.clone();
            let outgoing_tx = outgoing_tx.clone();
            let incoming_rx = incoming_rx.subscribe();
            async {
                Self::long_lived_watcher(incoming_rx, outgoing_tx, state).await;
            }
        });

        Self {
            state,
            outgoing_tx,
            connection_handle,
            incoming_rx,
        }
    }

    pub fn stop() {
        //
    }

    pub fn go_to_start() {
        //
    }

    pub async fn run_auto_script(&self, data: Vec<u8>) ->  broadcast::Receiver<TheBeginnerCarOutgoingMessages> {
        dbg!("running"); // TODO: this should error if not connected
        let mut rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages> = self.incoming_rx.subscribe();
        self.outgoing_tx
            .send(TheBeginnerCarIncomingMessages::StopAutoScript)
            .await
            .unwrap();

        dbg!("waiting for autoscript stopped message");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);

        loop {
            match timeout_at(deadline, rx.recv()).await {
                Ok(Ok(TheBeginnerCarOutgoingMessages::AutoScriptStopped)) => {
                    dbg!("got autoscript stopped message");
                    break;
                }
                Ok(Ok(_)) => {}
                Ok(Err(_)) => break,
                Err(_) => {
                    dbg!("timed out waiting for autoscript stopped message");
                    break;
                }
            }
        }

        dbg!("running auto script");

        let start_script = TheBeginnerCarIncomingMessages::RunAutoScript { data };

        self.outgoing_tx.send(start_script).await.unwrap();
        rx
    }

    pub fn queue_auto_script() {
        //
    }

    pub fn list_queued_auto_scripts() {
        //
    }

    // pub fn

    async fn long_lived_watcher(
        mut incoming_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
        outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
        state: Arc<Mutex<HardwareState>>,
    ) {
        loop {
            match incoming_rx.recv().await {
                Ok(message) => match message {
                    TheBeginnerCarOutgoingMessages::AutoScriptStopped => {
                        Self::on_auto_script_stopped(state.clone(), outgoing_tx.clone()).await;
                    }
                    _ => {}
                },
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    eprintln!("Car watcher lagged; skipped {skipped} messages");
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    }

    pub async fn on_auto_script_stopped(
        state: Arc<Mutex<HardwareState>>,
        outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
    ) {
        return; // TODO remove this
        let mut state = state.lock().await;
        let script = state.queued_scripts.pop().unwrap(); // TODO: this should be taking the next in the queue.......
        outgoing_tx
            .send(TheBeginnerCarIncomingMessages::RunAutoScript { data: script })
            .await
            .unwrap();
        //
    }

    // mut incoming_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
}
