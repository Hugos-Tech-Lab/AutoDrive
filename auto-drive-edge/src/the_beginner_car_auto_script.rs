use std::{collections::VecDeque, sync::{Arc, atomic::{AtomicU32, Ordering}}, time::Duration};

use sha2::{Digest, Sha256};
use the_beginner_car_tcp_protocol::{
    AutoScript, TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    sync::{
        Mutex, broadcast,
        mpsc::{self},
    },
    task::JoinHandle,
    time::timeout_at,
};
use uuid::Uuid;

static NEXT_RUN_NUMBER: AtomicU32 = AtomicU32::new(0);

pub struct AutoScriptState {
    queued_scripts: VecDeque<AutoScript>,
    running_script: Option<AutoScript>
}

pub struct TheBeginnerCarAutoScript {
    state: Arc<Mutex<AutoScriptState>>,
    pub outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
    pub incoming_rx: broadcast::Sender<TheBeginnerCarOutgoingMessages>,
    _connection_handle: JoinHandle<()>,
}

impl TheBeginnerCarAutoScript {
    pub fn new(
        outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
        incoming_rx: broadcast::Sender<TheBeginnerCarOutgoingMessages>,
    ) -> Self {
        let state = Arc::new(Mutex::new(AutoScriptState {
            queued_scripts: VecDeque::new(),
            running_script: None,
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
            _connection_handle: connection_handle,
            incoming_rx,
        }
    }

    pub fn stop() {
        //
    }

    pub async fn cancel_auto_script(&self, run_uuid: String) {
        self.outgoing_tx
            .send(TheBeginnerCarIncomingMessages::CancelAutoScript { run_uuid })
            .await
            .unwrap();
    }

    pub async fn queue_auto_script(
        &self,
        data: Vec<u8>,
    ) {
        let digest = Sha256::digest(&data);
        let digest = hex::encode(Sha256::digest(digest));
        let run_number = NEXT_RUN_NUMBER.fetch_add(1, Ordering::Relaxed);

        let auto_script = AutoScript {
            run_uuid: Uuid::new_v4().to_string(),
            preprocessed_file_hash: digest,
            run_number,
            script: data,
        };

        {
            let mut state = self.state.lock().await;
            state.queued_scripts.push_back(auto_script);
        }

        // // TODO: this should error if not connected to hardware
        // let mut rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages> =
        //     self.incoming_rx.subscribe();
        // self.outgoing_tx
        //     .send(TheBeginnerCarIncomingMessages::CancelAutoScript { run_uuid: () })
        //     .await
        //     .unwrap();
        // let deadline = tokio::time::Instant::now() + Duration::from_secs(5);

        // loop {
        //     match timeout_at(deadline, rx.recv()).await {
        //         Ok(Ok(TheBeginnerCarOutgoingMessages::AutoScriptStopped)) => {
        //             dbg!("got autoscript stopped message");
        //             break;
        //         }
        //         Ok(Ok(_)) => {}
        //         Ok(Err(_)) => break,
        //         Err(_) => {
        //             dbg!("timed out waiting for autoscript stopped message");
        //             break;
        //         }
        //     }
        // }

        // let start_script = TheBeginnerCarIncomingMessages::RunAutoScript { data };
        // self.outgoing_tx.send(start_script).await.unwrap();
        // rx
    }

    pub fn start_next_auto_script() {
        //
    }

    // pub fn queue_auto_script() {
    //     //
    // }

    pub fn list_queued_auto_scripts() {
        //
    }

    // pub fn

    async fn long_lived_watcher(
        mut incoming_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
        outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
        state: Arc<Mutex<AutoScriptState>>,
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
        state: Arc<Mutex<AutoScriptState>>,
        outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
    ) {
        return; // TODO remove this
        {
            let mut state = state.lock().await;
            let script = state.queued_scripts.pop_front().unwrap(); // TODO: this should be taking the next in the queue.......
        }

        // outgoing_tx
        //     .send(TheBeginnerCarIncomingMessages::RunAutoScript { data: script })
        //     .await
        //     .unwrap();
        //
    }

    // mut incoming_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
}
