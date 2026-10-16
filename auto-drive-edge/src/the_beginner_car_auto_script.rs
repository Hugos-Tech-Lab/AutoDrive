use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

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
};
use uuid::Uuid;

static NEXT_RUN_NUMBER: AtomicU32 = AtomicU32::new(0);

pub struct AutoScriptState {
    queued_scripts: VecDeque<AutoScript>,
    running_script: Option<AutoScript>,
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

    pub async fn enqueue_auto_script(&self, data: Vec<u8>) -> String {
        let digest = Sha256::digest(&data);
        let digest = hex::encode(Sha256::digest(digest));
        let run_number = NEXT_RUN_NUMBER.fetch_add(1, Ordering::Relaxed);
        let run_uuid = Uuid::new_v4().to_string();

        let auto_script = AutoScript {
            run_uuid: run_uuid.clone(),
            preprocessed_file_hash: digest,
            run_number,
            script: data,
        };

        {
            let mut state = self.state.lock().await;
            state.queued_scripts.push_back(auto_script);

            if let Some(next) = state.queued_scripts.pop_front() {
                state.running_script = Some(next.clone());
                self.outgoing_tx
                    .send(TheBeginnerCarIncomingMessages::RunAutoScript { script: next })
                    .await
                    .unwrap();
            }
        }

        // Self::start_next_auto_script_if_not_running(self.state.clone(), self.outgoing_tx.clone());
        run_uuid
    }

    // pub fn start_next_auto_script_if_not_running(
    //     state: Arc<Mutex<AutoScriptState>>,
    //     outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>,
    // ) {
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
        {
            let mut state = state.lock().await;
            if let Some(next) = state.queued_scripts.pop_front() {
                state.running_script = Some(next.clone());
                outgoing_tx
                    .send(TheBeginnerCarIncomingMessages::RunAutoScript { script: next })
                    .await
                    .unwrap();
            }
        }

        // outgoing_tx
        //     .send(TheBeginnerCarIncomingMessages::RunAutoScript { data: script })
        //     .await
        //     .unwrap();
        //
    }

    // mut incoming_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
}
