use std::{
    collections::VecDeque,
    sync::atomic::{AtomicU32, Ordering},
};

use auto_drive_common_types::LogMessage;
use the_beginner_car_tcp_protocol::TheBeginnerCarIncomingMessages;
use tokio::sync::{broadcast::Receiver, watch};

pub mod auto_script;
pub mod auto_script_on_vehicle_watcher;

use crate::{
    auto_script_manager::{
        auto_script::{AutoScript, AutoScriptType},
        auto_script_on_vehicle_watcher::AutoScriptOnVehicleWatcher,
    },
    the_beginner_car::TheBeginnerCar,
};

static NEXT_RUN_NUMBER: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug)]
pub enum AutoScriptStatus {
    NotReceivedByHardware,
    ReceivedByHardware,
    CallingMain,
    StoppedBySelf,
    CancelledBySafety,
    CancelledManually,
}

#[derive(Clone, Debug)]
pub enum AutoScriptEvents {
    LogMessage { log_message: LogMessage },
    ScriptExited,
}

pub struct AutoScriptManager {
    auto_script_on_vehicle_watcher: AutoScriptOnVehicleWatcher,
    vehicles: Vec<TheBeginnerCar>,
    scripts: watch::Sender<Vec<AutoScript>>,
}

impl AutoScriptManager {
    pub fn new(vehicles: Vec<TheBeginnerCar>) -> Self {
        let (scripts, _) = watch::channel::<AutoScripts>(AutoScripts::default());

        let auto_script_on_vehicle_watcher =
            AutoScriptOnVehicleWatcher::new(vehicles.clone(), scripts.clone());

        Self {
            auto_script_on_vehicle_watcher,
            vehicles,
            scripts,
        }
    }

    pub async fn cancel_auto_script(&self, run_uuid: String) {
        let hardware = self
            .scripts
            .borrow()
            .running
            .iter()
            .find(|script| script.run_uuid == run_uuid)
            .and_then(|script| script.hardware.clone());

        if let Some(hardware) = hardware {
            hardware
                .send(TheBeginnerCarIncomingMessages::CancelAutoScript {
                    run_uuid: run_uuid.clone(),
                })
                .await;
            return;
        }

        self.scripts.send_if_modified(|state| {
            let Some(pos) = state
                .queued
                .iter()
                .position(|script| script.run_uuid == run_uuid)
            else {
                return false;
            };

            if let Some(script) = state.queued.remove(pos) {
                state.stopped.push(script);
            }
            true
        });
    }

    pub async fn run_auto_script(&self, run_uuid: String, number_plate: String) -> Result<(), ()> {
        let Some(the_beginner_car) = self
            .vehicles
            .iter()
            .find(|car| car.metadata.number_plate == number_plate)
        else {
            return Err(());
        };

        let mut script = None;

        self.scripts.send_if_modified(|state| {
            let Some(auto_script) = state
                .queued
                .iter_mut()
                .find(|s| s.run_uuid == run_uuid && s.hardware.is_none())
            else {
                return false;
            };

            auto_script.hardware = Some(the_beginner_car.clone());
            script = Some(the_beginner_car_tcp_protocol::AutoScript {
                run_uuid: auto_script.run_uuid.clone(),
                preprocessed_file_hash: auto_script.uploaded_script_hash.clone(),
                run_number: auto_script.run_number,
                script: auto_script.uploaded_script.to_vec(),
            });
            true
        });

        if let Some(script) = script {
            the_beginner_car
                .send(TheBeginnerCarIncomingMessages::RunAutoScript { script })
                .await;
        }

        Ok(())
    }

    pub fn enqueue_auto_script(
        &self,
        owner_name: String,
        owner_comment: String,
        data: Vec<u8>,
        script_type: AutoScriptType,
    ) -> String {
        let run_number = NEXT_RUN_NUMBER.fetch_add(1, Ordering::Relaxed);

        let auto_script = AutoScript::new(
            owner_name,
            owner_comment,
            run_number,
            data.into(),
            script_type,
        );
        let auto_script_run_uuid = auto_script.run_uuid.clone();

        self.scripts.send_modify(|state| {
            state.queued.push_back(auto_script);
        });

        auto_script_run_uuid
    }

    pub async fn watch_auto_script_logs(
        &self,
        uuid: String,
    ) -> Option<(Vec<LogMessage>, Receiver<LogMessage>)> {
        let (logs_history, logs_live_rx) = {
            let state = self.scripts.borrow();
            let auto_script = state
                .queued
                .iter()
                .chain(state.running.iter())
                .find(|auto_script| auto_script.run_uuid == uuid)?;

            (
                auto_script.logs_history.clone(),
                auto_script.logs_live_tx.subscribe(),
            )
        };

        let logs_history = logs_history.read().await.clone();
        Some((logs_history, logs_live_rx))
    }

    pub fn watch_auto_script_overview(&self) -> (AutoScripts, watch::Receiver<AutoScripts>) {
        let mut receiver = self.scripts.subscribe();
        let snapshot = receiver.borrow_and_update().clone();
        (snapshot, receiver)
    }
}
