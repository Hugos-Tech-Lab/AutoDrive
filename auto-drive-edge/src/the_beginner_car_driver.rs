use std::sync::Arc;

use the_beginner_car_tcp_protocol::{
    TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    sync::{Mutex, broadcast, mpsc},
    task::JoinHandle,
};

struct HardwareState {
    queued_scripts: Vec<Vec<u8>>,
}

pub struct TheBeginnerCarDriver {
    state: Arc<Mutex<HardwareState>>,
    outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
    connection_handle: JoinHandle<()>,
}

impl TheBeginnerCarDriver {
    pub fn new(
        outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
        incoming_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
    ) -> Self {
        let state = Arc::new(Mutex::new(HardwareState {
            queued_scripts: vec![],
        }));

        let connection_handle = tokio::spawn({
            // let mdns_address = device_metadata.mdns_address.clone();
            // let incoming_tx = incoming_tx.clone();
            let state = state.clone();
            let outgoing_tx = outgoing_tx.clone();
            async {
                Self::long_lived_watcher(incoming_rx, outgoing_tx, state).await;
            }
        });

        Self {
            state,
            outgoing_tx,
            connection_handle,
        }
    }

    pub fn stop() {
        //
    }

    pub fn go_to_start() {
        //
    }

    pub fn start_auto_script(&self) {
        //
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
                        Self::on_auto_script_stopped(state.clone(), outgoing_tx.clone());
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

    pub async fn on_auto_script_stopped(state: Arc<Mutex<HardwareState>>, outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>) {
        let mut state = state.lock().await;
        let script = state.queued_scripts.pop().unwrap(); // TODO: this should be taking the next in the queue.......
        outgoing_tx.send(TheBeginnerCarIncomingMessages::RunAutoScript { data: script });
        //
    }

    // mut incoming_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
}
