use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use sha2::{Digest, Sha256};
use the_beginner_car_tcp_protocol::{
    TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    sync::{
        Mutex, broadcast,
        mpsc::{self},
    },
    task::JoinHandle,
};
use uuid::Uuid;

use crate::{devices_config::DeviceId, the_beginner_car_comms::TheBeginnerCarComms};

static NEXT_RUN_NUMBER: AtomicU32 = AtomicU32::new(0);

pub struct AutoScript {
    pub run_uuid: String,
    pub preprocessed_file_hash: String,
    pub run_number: u32,
    pub script: Vec<u8>,
    pub device: TheBeginnerCarComms,
}

pub struct AutoScriptTheBeginnerCars {
    the_beginner_cars: Vec<TheBeginnerCarComms>,
    queued_scripts: VecDeque<AutoScript>,
    running_scripts: Vec<AutoScript>,
}

pub struct AutoScriptManager {
    the_beginner_car_scripts: Arc<Mutex<AutoScriptTheBeginnerCars>>,
    connection_handles: Vec<JoinHandle<()>>,
}

impl AutoScriptManager {
    pub fn new(the_beginner_cars: Vec<TheBeginnerCarComms>) -> Self {
        let the_beginner_car_scripts = Arc::new(Mutex::new(AutoScriptTheBeginnerCars {
            queued_scripts: VecDeque::new(),
            running_scripts: Vec::new(),
            the_beginner_cars: the_beginner_cars.clone(),
        }));

        let connection_handles = the_beginner_cars
            .into_iter()
            .map(|the_beginner_car| {
                tokio::spawn({
                    let the_beginner_car_scripts = the_beginner_car_scripts.clone();
                    async {
                        Self::long_lived_watcher(the_beginner_car, the_beginner_car_scripts).await;
                    }
                })
            })
            .collect();

        Self {
            the_beginner_car_scripts,
            connection_handles,
        }
    }

    pub fn stop() {
        //
    }

    pub async fn cancel_auto_script(&self, run_uuid: String) {
        let the_beginner_car_scripts = self.the_beginner_car_scripts.lock().await;
        let Some(running_script) = the_beginner_car_scripts
            .running_scripts
            .iter()
            .find(|script| script.run_uuid == run_uuid)
        else {
            panic!(); // TODO: error no running script with that id
        };

        running_script
            .device
            .outgoing_tx
            .send(TheBeginnerCarIncomingMessages::CancelAutoScript { run_uuid })
            .await
            .unwrap();
    }

    pub async fn enqueue_auto_script(&self, data: Vec<u8>) -> String {
        let digest = Sha256::digest(&data);
        let digest = hex::encode(Sha256::digest(digest));
        let run_number = NEXT_RUN_NUMBER.fetch_add(1, Ordering::Relaxed);
        let run_uuid = Uuid::new_v4().to_string();

        {
            let mut state = self.the_beginner_car_scripts.lock().await;
            let the_beginner_car = state.the_beginner_cars.iter().find(|the_beginner_car|
                the_beginner_car.device_id == DeviceId::TcA
            ).unwrap();

            let auto_script = AutoScript {
                run_uuid: run_uuid.clone(),
                preprocessed_file_hash: digest,
                run_number,
                script: data,
                device: the_beginner_car.clone(),
            };

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

    pub fn list_queued_auto_scripts() {
        //
    }

    // pub fn

    async fn long_lived_watcher(
        the_beginner_car: TheBeginnerCarComms,
        state: Arc<Mutex<AutoScriptTheBeginnerCars>>,
    ) {
        loop {
            match the_beginner_car.incoming_tx.subscribe().recv().await {
                Ok(message) => match message {
                    TheBeginnerCarOutgoingMessages::AutoScriptStopped => {
                        Self::on_auto_script_stopped(
                            state.clone(),
                            the_beginner_car.outgoing_tx.clone(),
                        )
                        .await;
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
        state: Arc<Mutex<AutoScriptTheBeginnerCars>>,
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
