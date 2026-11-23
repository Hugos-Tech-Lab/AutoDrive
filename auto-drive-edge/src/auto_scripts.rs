use std::sync::atomic::{AtomicU32, Ordering};

use auto_drive_common_types::LogMessage;
use auto_drive_edge_protocol::{
    auto_script_lifecycle::AutoScriptLifecycle,
    auto_script_permission_level::AutoScriptPermissionLevel,
    auto_script_simplified_lifecycle::AutoScriptSimplifiedLifecycle,
};
use chrono::{DateTime, Utc};
use futures_util::Stream;
use log::warn;
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;

use crate::{auto_script::AutoScript, logs::Logs};

static NEXT_RUN_NUMBER: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug)]
pub struct AutoScripts {
    pub scripts: watch::Sender<Vec<AutoScript>>,
}

impl AutoScripts {
    pub fn new() -> Self {
        let (scripts, _) = watch::channel::<Vec<AutoScript>>(Vec::<AutoScript>::default());

        Self { scripts }
    }

    pub fn start_auto_script(&self) {
        //
    }

    pub async fn auto_script(&self, run_uuid: String) -> Option<AutoScript> {
        let auto_scripts = self.scripts.borrow();
        let auto_script = auto_scripts
            .iter()
            .find(|auto_script| auto_script.metadata.run_uuid == run_uuid)
            .map(|auto_script| auto_script.clone());

        auto_script
    }

    pub async fn cancel(&self, run_uuid: String) {
        self.scripts.send_if_modified(|auto_scripts| {
            let auto_script = auto_scripts
                .iter_mut()
                .find(|auto_script| auto_script.metadata.run_uuid == run_uuid);

            let Some(auto_script) = auto_script else {
                warn!("auto script not found");
                return false;
            };

            auto_script.metadata.lifecycle = AutoScriptLifecycle::Cancelling;
            true
        });
    }

    pub async fn notify_log(&self, run_uuid: String, log: LogMessage) {
        let mut logs: Option<Logs> = None;
        self.scripts.send_modify(|auto_scripts| {
            let auto_script = auto_scripts
                .iter_mut()
                .find(|auto_script| auto_script.metadata.run_uuid == run_uuid);

            let Some(auto_script) = auto_script else {
                warn!("auto script not found");
                return;
            };

            logs = Some(auto_script.payload.logs.clone());
        });

        if let Some(logs) = logs {
            logs.push(log).await;
        };
    }

    pub fn notify_vehicle_disconnected(&self) {
        // TODO: add vehicle ID
        self.scripts.send_modify(|auto_scripts| {
            for auto_script in auto_scripts {
                if auto_script.metadata.lifecycle.simplified()
                    == AutoScriptSimplifiedLifecycle::Driving
                {
                    auto_script.metadata.lifecycle = AutoScriptLifecycle::StoppedByDisconnect;
                }
            }
        });
    }

    pub fn notify_auto_script_started(
        &self,
        run_uuid: String,
        number_plate: String,
        script_start_time: Option<DateTime<Utc>>,
    ) {
        self.scripts.send_modify(|auto_scripts| {
            let auto_script = auto_scripts
                .iter_mut()
                .find(|auto_script| auto_script.metadata.run_uuid == run_uuid);

            let Some(auto_script) = auto_script else {
                warn!("auto script not found");
                return;
            };
            auto_script.metadata.lifecycle = AutoScriptLifecycle::StartedCallingMain;
            auto_script.metadata.script_start_time = script_start_time;
            auto_script.metadata.vehicle_number_plate = Some(number_plate);
        });
    }

    pub fn set_auto_script_state(
        &self,
        run_uuid: String,
        lifecycle: AutoScriptLifecycle,
        number_plate: String,
    ) {
        self.scripts.send_modify(|auto_scripts| {
            let auto_script = auto_scripts
                .iter_mut()
                .find(|auto_script| auto_script.metadata.run_uuid == run_uuid);

            let Some(auto_script) = auto_script else {
                warn!("auto script not found");
                return;
            };
            auto_script.metadata.lifecycle = lifecycle;
            auto_script.metadata.vehicle_number_plate = Some(number_plate);
        });
    }

    pub fn notify_auto_script_stopped(
        &self,
        run_uuid: String,
        number_plate: String,
        script_stop_time: Option<DateTime<Utc>>,
        time_trials_dnf: bool,
    ) {
        self.scripts.send_modify(|auto_scripts| {
            let auto_script = auto_scripts
                .iter_mut()
                .find(|auto_script| auto_script.metadata.run_uuid == *run_uuid);

            let Some(auto_script) = auto_script else {
                warn!("AutoScript not found ({run_uuid})");
                return;
            };

            if auto_script.metadata.script_stop_time.is_some() {
                warn!("Hardware stopped a second time ({run_uuid})");
                return;
            }

            auto_script
                .metadata
                .vehicle_number_plate
                .get_or_insert(number_plate);
            auto_script.metadata.script_stop_time = script_stop_time;

            if auto_script.metadata.script_stop_time.is_some() {
                warn!("Stop time is unexpectedly some");
            }

            if auto_script.metadata.time_trials_dnf.is_some() {
                warn!("DNF is unexpectedly some");
            }

            auto_script.metadata.time_trials_dnf = Some(time_trials_dnf);
        });
    }

    pub fn enqueue_auto_script(
        &self,
        owner_name: String,
        owner_comment: String,
        data: Vec<u8>,
        permission_level: AutoScriptPermissionLevel,
    ) -> String {
        let run_number = NEXT_RUN_NUMBER.fetch_add(1, Ordering::Relaxed);

        let auto_script = AutoScript::new(
            owner_name,
            owner_comment,
            run_number,
            data.into(),
            permission_level,
        );

        let run_uuid = auto_script.metadata.run_uuid.clone();

        self.scripts.send_modify(|state| {
            state.push(auto_script);
        });

        run_uuid
    }

    pub fn stream(&self) -> impl Stream<Item = Vec<AutoScript>> + Send + Sync + 'static {
        WatchStream::new(self.scripts.subscribe().clone())
    }
}

// pub struct AutoScriptManager {
//     vehicles: Vec<TheBeginnerCar>,
//     scripts: watch::Sender<Vec<AutoScript>>,
// }

// impl AutoScriptManager {
//     pub fn new(vehicles: Vec<TheBeginnerCar>) -> Self {
//         let (scripts, _) = watch::channel::<AutoScripts>(AutoScripts::default());

//         let auto_script_on_vehicle_watcher =
//             AutoScriptOnVehicleWatcher::new(vehicles.clone(), scripts.clone());

//         Self { vehicles, scripts }
//     }

//     pub async fn cancel_auto_script(&self, run_uuid: String) {
//         let hardware = self
//             .scripts
//             .borrow()
//             .running
//             .iter()
//             .find(|script| script.run_uuid == run_uuid)
//             .and_then(|script| script.hardware.clone());

//         if let Some(hardware) = hardware {
//             hardware
//                 .send(TheBeginnerCarIncomingMessages::CancelAutoScript {
//                     run_uuid: run_uuid.clone(),
//                 })
//                 .await;
//             return;
//         }

//         self.scripts.send_if_modified(|state| {
//             let Some(pos) = state
//                 .queued
//                 .iter()
//                 .position(|script| script.run_uuid == run_uuid)
//             else {
//                 return false;
//             };

//             if let Some(script) = state.queued.remove(pos) {
//                 state.stopped.push(script);
//             }
//             true
//         });
//     }

//     pub async fn run_auto_script(&self, run_uuid: String, number_plate: String) -> Result<(), ()> {
//         let Some(the_beginner_car) = self
//             .vehicles
//             .iter()
//             .find(|car| car.metadata.number_plate == number_plate)
//         else {
//             return Err(());
//         };

//         let mut script = None;

//         self.scripts.send_if_modified(|state| {
//             let Some(auto_script) = state
//                 .queued
//                 .iter_mut()
//                 .find(|s| s.run_uuid == run_uuid && s.hardware.is_none())
//             else {
//                 return false;
//             };

//             auto_script.hardware = Some(the_beginner_car.clone());
//             script = Some(the_beginner_car_tcp_protocol::AutoScript {
//                 run_uuid: auto_script.run_uuid.clone(),
//                 preprocessed_file_hash: auto_script.uploaded_script_hash.clone(),
//                 run_number: auto_script.run_number,
//                 script: auto_script.uploaded_script.to_vec(),
//             });
//             true
//         });

//         if let Some(script) = script {
//             the_beginner_car
//                 .send(TheBeginnerCarIncomingMessages::RunAutoScript { script })
//                 .await;
//         }

//         Ok(())
//     }

//     pub async fn watch_auto_script_logs(
//         &self,
//         uuid: String,
//     ) -> Option<(Vec<LogMessage>, Receiver<LogMessage>)> {
//         let (logs_history, logs_live_rx) = {
//             let state = self.scripts.borrow();
//             let auto_script = state
//                 .queued
//                 .iter()
//                 .chain(state.running.iter())
//                 .find(|auto_script| auto_script.run_uuid == uuid)?;

//             (
//                 auto_script.logs_history.clone(),
//                 auto_script.logs_live_tx.subscribe(),
//             )
//         };

//         let logs_history = logs_history.read().await.clone();
//         Some((logs_history, logs_live_rx))
//     }

//     pub fn watch_auto_script_overview(&self) -> (AutoScripts, watch::Receiver<AutoScripts>) {
//         let mut receiver = self.scripts.subscribe();
//         let snapshot = receiver.borrow_and_update().clone();
//         (snapshot, receiver)
//     }
// }
