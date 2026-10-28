use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use auto_drive_common_types::{LogLevel, LogMessage};
use log::{info, warn};
use the_beginner_car_tcp_protocol::{
    TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    sync::{
        Mutex,
        broadcast::{self, Receiver, Sender},
    },
    task::JoinHandle,
};

pub mod auto_script;

use crate::{
    auto_script_manager::auto_script::AutoScript, the_beginner_car_comms::TheBeginnerCarComms,
};

static NEXT_RUN_NUMBER: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug)]
pub enum AutoScriptResponseFromHardware {
    HeardNothing,
    Received,
    CallingMain,
    Error,
}

#[derive(Clone, Debug)]
pub enum AutoScriptEvents {
    LogMessage { log_message: LogMessage },
    ScriptExited,
}

// #[derive(Debug, Clone)]
// pub struct AutoScriptSnapshot {
//     pub owner_name: String,
//     pub queued_at: DateTime<Utc>,
//     pub started_at: Option<DateTime<Utc>>,
//     pub stopped_at: Option<DateTime<Utc>>,
//     pub run_uuid: String,
//     pub upload_file_hash: String,
// }

#[derive(Clone, Debug, Default)]
pub struct AutoScripts {
    pub running: Vec<AutoScript>,
    pub queued: VecDeque<AutoScript>,
    pub stopped: Vec<AutoScript>,
}

pub struct AutoScriptTheBeginnerCars {
    the_beginner_cars: Vec<TheBeginnerCarComms>,
    queue_events_sender: Sender<AutoScripts>,
    pub scripts: AutoScripts,
}

pub struct AutoScriptManager {
    the_beginner_car_scripts: Arc<Mutex<AutoScriptTheBeginnerCars>>,
    connection_handles: Vec<JoinHandle<()>>,
}

impl AutoScriptManager {
    pub fn new(the_beginner_cars: Vec<TheBeginnerCarComms>) -> Self {
        let (sender, _) = broadcast::channel::<AutoScripts>(100);

        let the_beginner_car_scripts = Arc::new(Mutex::new(AutoScriptTheBeginnerCars {
            the_beginner_cars: the_beginner_cars.clone(),
            scripts: AutoScripts::default(),
            queue_events_sender: sender,
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
        let queue_snapshot: AutoScripts;
        let queue_events_sender: Sender<AutoScripts>;
        {
            let mut the_beginner_car_scripts = self.the_beginner_car_scripts.lock().await;
            if let Some(running_script) = the_beginner_car_scripts
                .scripts
                .running
                .iter()
                .find(|script| script.run_uuid == run_uuid)
            {
                running_script
                    .hardware
                    .send(TheBeginnerCarIncomingMessages::CancelAutoScript {
                        run_uuid: run_uuid.clone(),
                    })
                    .await;
            }

            if let Some(pos) = the_beginner_car_scripts
                .scripts
                .queued
                .iter()
                .position(|script| script.run_uuid == run_uuid)
            {
                if let Some(script) = the_beginner_car_scripts.scripts.queued.remove(pos) {
                    the_beginner_car_scripts.scripts.stopped.push(script);
                } else {
                    // TODO: print warning
                }

                queue_snapshot = the_beginner_car_scripts.scripts.clone();
                queue_events_sender = the_beginner_car_scripts.queue_events_sender.clone();
            } else {
                return;
            }
        }

        queue_events_sender.send(queue_snapshot).unwrap();
    }

    pub async fn enqueue_auto_script(
        &self,
        owner_name: String,
        owner_comment: String,
        data: Vec<u8>,
    ) -> String {
        let run_number = NEXT_RUN_NUMBER.fetch_add(1, Ordering::Relaxed);
        let queue_snapshot: AutoScripts;
        let queue_events_sender: Sender<AutoScripts>;
        let auto_script_run_uuid: String;
        {
            let mut state = self.the_beginner_car_scripts.lock().await;
            let the_beginner_car = state
                .the_beginner_cars
                .iter()
                .find(|the_beginner_car| the_beginner_car.number_plate == "TBC 001".to_string())
                .unwrap();

            let auto_script = AutoScript::new(
                owner_name,
                owner_comment,
                run_number,
                data.into(),
                the_beginner_car.clone(),
            );

            auto_script_run_uuid = auto_script.run_uuid.clone();

            queue_snapshot = state.scripts.clone();
            queue_events_sender = state.queue_events_sender.clone();
            state.scripts.queued.push_back(auto_script);
        }
        let _ = queue_events_sender.send(queue_snapshot);
        Self::try_start_next_auto_script(self.the_beginner_car_scripts.clone()).await;
        auto_script_run_uuid
    }

    pub async fn watch_auto_script(
        &self,
        uuid: String,
    ) -> Option<(Vec<AutoScriptEvents>, Receiver<AutoScriptEvents>)> {
        let the_beginner_car_scripts_mutex_guard = self.the_beginner_car_scripts.lock().await;
        let auto_script = the_beginner_car_scripts_mutex_guard
            .scripts
            .queued
            .iter()
            .chain(the_beginner_car_scripts_mutex_guard.scripts.running.iter())
            .find(|asd| asd.run_uuid == uuid);

        if let Some(auto_script) = auto_script {
            let hist_lock = auto_script.events_history.read().await;
            let past_events = hist_lock.clone();
            let live_rx = auto_script.events_tx.subscribe();
            return Some((past_events, live_rx));
        }

        // TODO: also check stopped to provide a clearer error message
        None
    }

    pub async fn watch_auto_script_overview(&self) -> (AutoScripts, Receiver<AutoScripts>) {
        let the_beginner_car_scripts_mutex_guard = self.the_beginner_car_scripts.lock().await;
        let snapshot = the_beginner_car_scripts_mutex_guard.scripts.clone();

        let receiver = the_beginner_car_scripts_mutex_guard
            .queue_events_sender
            .subscribe();

        (snapshot, receiver)
    }

    pub async fn try_start_next_auto_script(
        the_beginner_car_scripts: Arc<Mutex<AutoScriptTheBeginnerCars>>,
    ) {
        let target_device_id = "TBC 001".to_string(); // In the future, find available device.

        let target_device: TheBeginnerCarComms;
        let script_to_start_content;
        let queue_snapshot: AutoScripts;
        let queue_events_sender: Sender<AutoScripts>;
        let target_device_incoming_tx: broadcast::Sender<TheBeginnerCarOutgoingMessages>; // TODO: hack to easily send logs

        {
            let mut the_beginner_car_scripts_mutex_guard = the_beginner_car_scripts.lock().await;
            let device_being_used = the_beginner_car_scripts_mutex_guard
                .scripts
                .running
                .iter()
                .any(|auto_script| auto_script.hardware.number_plate == target_device_id);
            if device_being_used {
                // cannot start because it's already being used..
                for (i, script) in the_beginner_car_scripts_mutex_guard
                    .scripts
                    .queued
                    .iter()
                    .enumerate()
                {
                    script
                        .hardware
                        .incoming_tx
                        .send(
                            the_beginner_car_tcp_protocol::TheBeginnerCarOutgoingMessages::Log {
                                log: LogMessage {
                                    system_origin:
                                        auto_drive_common_types::LogSystemOrigin::EdgeServer,
                                    level: LogLevel::Info,
                                    target: "asd".to_string(),
                                    // trace_id: Some(script.run_uuid.clone()),
                                    message: format!("You are waiting in the queue, position {i}"),
                                    trace_id: None,
                                },
                            },
                        )
                        .unwrap();
                }

                return;
            }

            let script_to_start = the_beginner_car_scripts_mutex_guard
                .scripts
                .queued
                .pop_front();

            if let Some(script) = script_to_start {
                if !script.hardware.is_connected() {
                    script
                        .send_log_info(
                            "Cannot start script because hardware is not connected.".to_string(),
                        )
                        .await;
                    the_beginner_car_scripts_mutex_guard
                        .scripts
                        .queued
                        .push_front(script);

                    return;
                }

                target_device = script.hardware.clone();
                script_to_start_content = the_beginner_car_tcp_protocol::AutoScript {
                    run_uuid: script.run_uuid.clone(),
                    preprocessed_file_hash: script.uploaded_script_hash.clone(),
                    run_number: script.run_number,
                    device_id: "script.device.device_id".to_string(),
                    script: script.uploaded_script.to_vec(),
                };

                script
                    .send_log_info("Sending script run command to hardware.".to_string())
                    .await;

                // target_device_incoming_tx = script
                //     .device
                //     .incoming_tx.clone();

                the_beginner_car_scripts_mutex_guard
                    .scripts
                    .running
                    .push(script);
            } else {
                // cannot start because no script is available..
                return;
            }

            queue_snapshot = AutoScripts {
                running: the_beginner_car_scripts_mutex_guard.scripts.running.clone(), // TODO: make consistent this variable name with other functions usage: the_beginner_car_scripts_mutex_guard
                queued: the_beginner_car_scripts_mutex_guard
                    .scripts
                    .queued
                    .clone()
                    .into(),
                stopped: the_beginner_car_scripts_mutex_guard.scripts.stopped.clone(),
            };
            queue_events_sender = the_beginner_car_scripts_mutex_guard
                .queue_events_sender
                .clone();
        }

        // error when no listener
        let _ = queue_events_sender.send(queue_snapshot);
        let sent = target_device
            .send(TheBeginnerCarIncomingMessages::RunAutoScript {
                script: script_to_start_content,
            })
            .await;

        if !sent {
            dbg!("not sent");
            //
        }
    }

    pub fn list_queued_auto_scripts() {
        //
    }

    // pub fn

    async fn long_lived_watcher(
        the_beginner_car: TheBeginnerCarComms,
        state: Arc<Mutex<AutoScriptTheBeginnerCars>>,
    ) {
        let mut incoming_rx = the_beginner_car.incoming_tx.subscribe();

        loop {
            match incoming_rx.recv().await {
                Ok(message) => match message {
                    TheBeginnerCarOutgoingMessages::AutoScriptStopped => {
                        Self::on_hardware_response_auto_script_stopped(
                            state.clone(),
                            the_beginner_car.number_plate.clone(),
                        )
                        .await;
                    }
                    _ => {}
                },
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    warn!("Car watcher lagged; skipped {skipped} messages");
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("AutoScript long lived watcher closed");
                    break;
                }
            }
        }
    }

    pub async fn on_hardware_response_auto_script_stopped(
        state: Arc<Mutex<AutoScriptTheBeginnerCars>>,
        number_plate: String,
    ) {
        let queue_snapshot: AutoScripts;
        let queue_events_sender: Sender<AutoScripts>;
        {
            let mut state_mutex_guard = state.lock().await;
            let stopped: Vec<AutoScript> = state_mutex_guard
                .scripts
                .running
                .extract_if(.., |auto_script| {
                    auto_script.hardware.number_plate == number_plate
                })
                .collect();
            if stopped.len() != 1 {
                // should only be one.. but ah well maybe we got here somehow..
                warn!("Unexpected number of scripts removed: ({})", stopped.len());
            }

            for stopped_script in stopped {
                stopped_script
                    .send_log_info("Script exited".to_string())
                    .await;

                state_mutex_guard.scripts.stopped.push(stopped_script);
            }

            queue_snapshot = state_mutex_guard.scripts.clone();
            queue_events_sender = state_mutex_guard.queue_events_sender.clone();
        }

        queue_events_sender.send(queue_snapshot).unwrap();

        Self::try_start_next_auto_script(state).await;
    }
}
