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
        Mutex,
        broadcast::{self, Receiver},
        mpsc::{self},
    },
    task::JoinHandle,
};
use uuid::Uuid;

use crate::{devices_config::DeviceId, the_beginner_car_comms::TheBeginnerCarComms};

static NEXT_RUN_NUMBER: AtomicU32 = AtomicU32::new(0);

pub enum AutoScriptResponseFromHardware {
    HeardNothing,
    Received,
    CallingMain,
    Error,
}

#[derive(Clone, Debug)]
pub enum AutoScriptEvents {
    Log,
}

pub struct AutoScript {
    pub run_uuid: String,
    pub preprocessed_file_hash: String,
    pub run_number: u32,
    pub script: Vec<u8>,
    pub events: Receiver<AutoScriptEvents>,
    pub device: TheBeginnerCarComms,
    pub hardware_response: AutoScriptResponseFromHardware,
}

pub struct AutoScriptTheBeginnerCars {
    the_beginner_cars: Vec<TheBeginnerCarComms>,
    queued_scripts: VecDeque<AutoScript>,
    running_scripts: Vec<AutoScript>,
    stopped_scripts: Vec<AutoScript>,
}

pub struct AutoScriptManager {
    the_beginner_car_scripts: Arc<Mutex<AutoScriptTheBeginnerCars>>,
    connection_handles: Vec<JoinHandle<()>>,
}

impl AutoScriptManager {
    pub fn new(the_beginner_cars: Vec<TheBeginnerCarComms>) -> Self {
        let the_beginner_car_scripts = Arc::new(Mutex::new(AutoScriptTheBeginnerCars {
            the_beginner_cars: the_beginner_cars.clone(),
            queued_scripts: VecDeque::new(),
            running_scripts: Vec::new(),
            stopped_scripts: Vec::new(),
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
            let the_beginner_car = state
                .the_beginner_cars
                .iter()
                .find(|the_beginner_car| the_beginner_car.device_id == DeviceId::TcA)
                .unwrap();

            let (sender, receiver) = broadcast::channel(100);

            let auto_script = AutoScript {
                run_uuid: run_uuid.clone(),
                preprocessed_file_hash: digest,
                run_number,
                script: data,
                device: the_beginner_car.clone(),
                hardware_response: AutoScriptResponseFromHardware::HeardNothing,
                events: receiver,
            };

            // TODO: AutoScript should hold onto the join_handle
            let join_handle = tokio::task::spawn({
                // TODO: should listen to cancellation token and stop when cancelled
                let mut incoming_tx = the_beginner_car.incoming_tx.subscribe();
                let run_uuid = run_uuid.clone();
                async move {
                    match incoming_tx.recv().await {
                        Ok(message) => match message {
                            TheBeginnerCarOutgoingMessages::Log { log } => {
                                if log.trace_id.is_some_and(|trace_id| trace_id == run_uuid) {
                                    //
                                    sender.send(AutoScriptEvents::Log).unwrap(); // TODO: send the log
                                }
                            }
                            _ => {}
                        },
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            eprintln!("Car watcher lagged; skipped {skipped} messages");
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            // break;
                        }
                    };
                }
            });

            state.queued_scripts.push_back(auto_script);
        }

        // Self::start_next_auto_script_if_not_running(self.state.clone(), self.outgoing_tx.clone());
        run_uuid
    }

    pub async fn watch_auto_script(&self, uuid: String) -> Option<Receiver<AutoScriptEvents>> {
        let the_beginner_car_scripts_mutex_guard = self.the_beginner_car_scripts.lock().await;
        let auto_script = the_beginner_car_scripts_mutex_guard
            .queued_scripts
            .iter()
            .find(|asd| asd.run_uuid == uuid);
        if let Some(auto_script) = auto_script {
            return Some(auto_script.events.resubscribe());
        }
        let auto_script = the_beginner_car_scripts_mutex_guard
            .running_scripts
            .iter()
            .find(|asd| asd.run_uuid == uuid);
        if let Some(auto_script) = auto_script {
            return Some(auto_script.events.resubscribe());
        }

        // TODO: also check stopped to provide a clearer error message
        None
    }

    pub async fn try_start_next_auto_script(
        the_beginner_car_scripts: Arc<Mutex<AutoScriptTheBeginnerCars>>,
    ) {
        let target_device_id = DeviceId::TcA; // In the future, find available device.

        let target_device_outgoing_tx: mpsc::Sender<TheBeginnerCarIncomingMessages>;
        let script_to_start_content;
        {
            let mut the_beginner_car_scripts_mutex_guard = the_beginner_car_scripts.lock().await;
            let device_being_used = the_beginner_car_scripts_mutex_guard
                .running_scripts
                .iter()
                .any(|auto_script| auto_script.device.device_id == target_device_id);
            if device_being_used {
                panic!(); // TODO: proper error
            }

            let script_to_start = the_beginner_car_scripts_mutex_guard
                .queued_scripts
                .pop_front();

            if let Some(script) = script_to_start {
                target_device_outgoing_tx = script.device.outgoing_tx.clone();
                script_to_start_content = the_beginner_car_tcp_protocol::AutoScript {
                    run_uuid: script.run_uuid.clone(),
                    preprocessed_file_hash: script.preprocessed_file_hash.clone(),
                    run_number: script.run_number,
                    device_id: "script.device.device_id".to_string(),
                    script: script.script.clone(),
                };

                the_beginner_car_scripts_mutex_guard
                    .running_scripts
                    .push_mut(script);
            } else {
                panic!();
            }
        }

        target_device_outgoing_tx
            .send(TheBeginnerCarIncomingMessages::RunAutoScript {
                script: script_to_start_content,
            })
            .await
            .unwrap();
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
                        Self::on_hardware_response_auto_script_stopped(
                            state.clone(),
                            the_beginner_car.device_id.clone(),
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

    pub async fn on_hardware_response_auto_script_stopped(
        state: Arc<Mutex<AutoScriptTheBeginnerCars>>,
        device_id: DeviceId,
    ) {
        {
            let mut state_mutex_guard = state.lock().await;
            let stopped_scripts: Vec<AutoScript> = state_mutex_guard
                .running_scripts
                .extract_if(.., |auto_script| auto_script.device.device_id == device_id)
                .collect();
            if stopped_scripts.len() != 1 {
                // should only be one.. but ah well maybe we got here somehow..
                println!(
                    "warn: unexpected number of scripts removed: ({})",
                    stopped_scripts.len()
                );
            }

            for removed_script in stopped_scripts {
                state_mutex_guard.stopped_scripts.push(removed_script);
            }
        }

        Self::try_start_next_auto_script(state);
    }

    // mut incoming_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
}
