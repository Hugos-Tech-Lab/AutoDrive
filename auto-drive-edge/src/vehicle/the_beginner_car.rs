use std::sync::Arc;

use chrono::{DateTime, Utc};
use hardware_config::TheBeginnerCarMetadata;
use log::{error, info, warn};
use the_beginner_car_tcp_protocol::{
    TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    sync::{broadcast, watch},
    task::JoinHandle,
};

use crate::{
    auto_script_manager::{AutoScriptLifecycle, AutoScripts, auto_script::AutoScript},
    vehicle::the_beginner_car::the_beginner_car_connection_handle::TheBeginnerCarConnectionHandle,
};

pub mod the_beginner_car_auto_script_watcher;
pub mod the_beginner_car_connection_handle;

enum TheBeginnerCarAutoScriptStatus {
    Ready,
    Sending,
}

#[derive(Clone, Debug)]
pub struct TheBeginnerCar {
    metadata: TheBeginnerCarMetadata,
    connection: TheBeginnerCarConnectionHandle,
    scripts: AutoScripts,
    handle_messages_from_vehicle_task_handle: Arc<JoinHandle<()>>,
    logs: (),
    vehicle_lifecycle: watch::Sender<AutoScriptLifecycle>,
    handle_vehicle_connection_handle: Arc<JoinHandle<()>>,
}

impl TheBeginnerCar {
    pub fn new(metadata: TheBeginnerCarMetadata, scripts: AutoScripts) -> Self {
        let connection = TheBeginnerCarConnectionHandle::new(metadata.clone());

        let vehicle_lifecycle =
            watch::Sender::<AutoScriptLifecycle>::new(AutoScriptLifecycle::Idle);

        let handle_messages_from_vehicle_task_handle =
            tokio::spawn(handle_messages_from_vehicle_task(
                connection.clone(),
                scripts.clone(),
                vehicle_lifecycle.clone(),
                metadata.number_plate.to_string(),
            ));

        let handle_vehicle_connection_handle = tokio::spawn(handle_vehicle_connection(
            connection.clone(),
            scripts.clone(),
        ));

        Self {
            metadata,
            connection,
            handle_messages_from_vehicle_task_handle: Arc::new(
                handle_messages_from_vehicle_task_handle,
            ),
            logs: (),
            vehicle_lifecycle,
            scripts,
            handle_vehicle_connection_handle: Arc::new(handle_vehicle_connection_handle),
        }
    }

    pub fn number_plate(&self) -> String {
        self.metadata.number_plate.to_string()
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_connected()
    }

    pub async fn send(&self, message: TheBeginnerCarIncomingMessages) -> bool {
        self.connection.send(message).await
    }

    pub async fn start_auto_script(&self, run_uuid: String) {
        let mut auto_script_to_start: Option<AutoScript> = None;

        let modified = self.scripts.scripts.send_if_modified(|auto_scripts| {
            let active_auto_script = auto_scripts
                .iter()
                .find(|auto_script| auto_script.auto_script_lifecycle.is_active());

            if active_auto_script.is_some() {
                warn!("other auto script is active");
                return false;
            }

            let auto_script = auto_scripts
                .iter_mut()
                .find(|auto_script| auto_script.run_uuid == run_uuid);

            let Some(auto_script) = auto_script else {
                warn!("auto script not found");
                return false;
            };

            auto_script.auto_script_lifecycle = AutoScriptLifecycle::SendingToWifiMcu;
            auto_script_to_start = Some(auto_script.clone());
            true
        });

        if !modified {
            return;
        }

        let Some(auto_script_to_start) = auto_script_to_start else {
            error!(
                "Auto Script to start was not set, despite modified being true, state is out of sync!"
            );
            return;
        };

        let vehicle_sending_wifi_mcu =
            self.vehicle_lifecycle
                .send_if_modified(|vehicle_lifecycle| {
                    if vehicle_lifecycle.is_active() {
                        return false;
                    }
                    *vehicle_lifecycle = AutoScriptLifecycle::SendingToWifiMcu;
                    true
                });

        if vehicle_sending_wifi_mcu {
            warn!("Vehicle is in use, cannot start auto script");
            self.scripts
                .set_auto_script_state(run_uuid, AutoScriptLifecycle::Idle, "".to_string());
            return;
        }

        let sent = self
            .connection
            .send(TheBeginnerCarIncomingMessages::RunAutoScript {
                script: the_beginner_car_tcp_protocol::AutoScript {
                    run_uuid: run_uuid.clone(),
                    preprocessed_file_hash: auto_script_to_start.uploaded_script_hash,
                    run_number: auto_script_to_start.run_number,
                    script: auto_script_to_start.uploaded_script.to_vec(),
                },
            })
            .await;

        if !sent {
            // revert vehicle lifecycle state
            self.vehicle_lifecycle
                .send_replace(AutoScriptLifecycle::Idle);
            self.scripts.set_auto_script_state(
                run_uuid,
                AutoScriptLifecycle::Idle,
                self.number_plate(),
            );
        }
    }
}

pub async fn handle_vehicle_connection(
    mut connection: TheBeginnerCarConnectionHandle,
    scripts: AutoScripts,
) {
    while connection.connected_rx.changed().await.is_ok() {
        let is_connected = *connection.connected_rx.borrow();
        if !is_connected {
            scripts.notify_vehicle_disconnected();
        }
    }

    info!("Stopped handling vehicle connection");
}

pub async fn handle_messages_from_vehicle_task(
    mut connection: TheBeginnerCarConnectionHandle,
    scripts: AutoScripts,
    vehicle_lifecycle: watch::Sender<AutoScriptLifecycle>,
    number_plate: String,
) {
    loop {
        match connection.incoming_rx.recv().await {
            Ok(message) => {
                handle_message_from_vehicle(
                    message,
                    scripts.clone(),
                    vehicle_lifecycle.clone(),
                    number_plate,
                );
            }
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                warn!("Car watcher lagged; skipped {skipped} messages");
            }
            Err(broadcast::error::RecvError::Closed) => {
                info!("Car long lived watcher closed");
                break;
            }
        }
    }
}

pub async fn handle_message_from_vehicle(
    message: TheBeginnerCarOutgoingMessages,
    scripts: AutoScripts,
    vehicle_lifecycle: watch::Sender<AutoScriptLifecycle>,
    number_plate: String,
) {
    match message {
        TheBeginnerCarOutgoingMessages::Log { log } => {
            //
        }
        TheBeginnerCarOutgoingMessages::AutoScriptReceivedByWifiMcu { run_uuid } => {
            scripts.set_auto_script_state(
                run_uuid,
                AutoScriptLifecycle::ReceivedByWifiMcu,
                number_plate,
            );
            vehicle_lifecycle.send_replace(AutoScriptLifecycle::ReceivedByWifiMcu);
        }

        TheBeginnerCarOutgoingMessages::AutoScriptSendingToWasmMcu { run_uuid } => {
            scripts.set_auto_script_state(
                run_uuid,
                AutoScriptLifecycle::SendingToWasmMcu,
                number_plate,
            );
            vehicle_lifecycle.send_replace(AutoScriptLifecycle::SendingToWasmMcu);
        }
        TheBeginnerCarOutgoingMessages::AutoScriptReceivedWasmMcu { run_uuid } => {
            scripts.set_auto_script_state(
                run_uuid,
                AutoScriptLifecycle::ReceivedByWasmMcu,
                number_plate,
            );
            vehicle_lifecycle.send_replace(AutoScriptLifecycle::ReceivedByWasmMcu);
        }
        TheBeginnerCarOutgoingMessages::AutoScriptStartedCallingMain {
            run_uuid,
            script_start_time_utc_ms,
        } => {
            let script_start_time_utc =
                DateTime::<Utc>::from_timestamp_millis(script_start_time_utc_ms as i64);
            if script_start_time_utc.is_none() {
                warn!(
                    "Script start time is none when creating from u64 ({script_start_time_utc_ms})"
                )
            }
            scripts.notify_auto_script_started(run_uuid, number_plate, script_start_time_utc);
            vehicle_lifecycle.send_replace(AutoScriptLifecycle::StartedCallingMain);
        }
        TheBeginnerCarOutgoingMessages::AutoScriptStopped {
            run_uuid,
            script_stop_time_utc_ms,
            time_trials_dnf,
        } => {
            let script_stop_time_utc =
                DateTime::<Utc>::from_timestamp_millis(script_stop_time_utc_ms as i64);
            if script_stop_time_utc.is_none() {
                warn!("Script stop time is none when creating from u64 ({script_stop_time_utc_ms})")
            }
            scripts.notify_auto_script_stopped(
                run_uuid,
                number_plate,
                script_stop_time_utc,
                time_trials_dnf,
            );
            vehicle_lifecycle.send_replace(AutoScriptLifecycle::StartedCallingMain);
        }
        TheBeginnerCarOutgoingMessages::AutoScriptTimeTrialsStarted {
            vehicle_start_time_utc_ms,
        } => todo!(),
        TheBeginnerCarOutgoingMessages::AutoScriptTimeTrialsFinished {
            vehicle_finish_time_utc_ms,
        } => todo!(),
        TheBeginnerCarOutgoingMessages::AutoScriptRejected { run_uuid } => todo!(),
        TheBeginnerCarOutgoingMessages::BatteryReading { voltage } => todo!(),
        TheBeginnerCarOutgoingMessages::InfraredLedReadings {
            front_far_left,
            front_left,
            front_center,
            front_right,
            front_far_right,
            back_center,
            back_left,
        } => todo!(),
        TheBeginnerCarOutgoingMessages::Position {
            x,
            y,
            rotation_radians,
            timestamp_unix_epoch_ms,
        } => todo!(),
    }
}
