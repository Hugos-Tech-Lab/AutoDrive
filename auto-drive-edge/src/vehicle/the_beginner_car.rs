use std::sync::Arc;

use hardware_config::TheBeginnerCarMetadata;
use log::{info, warn};
use the_beginner_car_tcp_protocol::{
    TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    sync::{broadcast, watch},
    task::JoinHandle,
};

use crate::{
    auto_script_manager::auto_script::AutoScript,
    vehicle::the_beginner_car::the_beginner_car_connection_handle::TheBeginnerCarConnectionHandle,
};

pub mod the_beginner_car_auto_script_watcher;
pub mod the_beginner_car_connection_handle;

#[derive(Clone, Debug)]
pub struct TheBeginnerCar {
    connection: TheBeginnerCarConnectionHandle,
    handle_messages_from_vehicle_task_handle: Arc<JoinHandle<()>>,
}

impl TheBeginnerCar {
    pub fn new(
        device_metadata: TheBeginnerCarMetadata,
        scripts: watch::Sender<Vec<AutoScript>>,
    ) -> Self {
        let connection = TheBeginnerCarConnectionHandle::new(device_metadata);

        let handle_messages_from_vehicle_task_handle = tokio::spawn(
            handle_messages_from_vehicle_task(connection.clone(), scripts),
        );

        Self {
            connection,
            handle_messages_from_vehicle_task_handle: Arc::new(
                handle_messages_from_vehicle_task_handle,
            ),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_connected()
    }

    pub async fn send(&self, message: TheBeginnerCarIncomingMessages) -> bool {
        self.connection.send(message).await
    }
}

pub async fn handle_messages_from_vehicle_task(
    mut connection: TheBeginnerCarConnectionHandle,
    scripts: watch::Sender<Vec<AutoScript>>,
) {
    loop {
        match connection.incoming_rx.recv().await {
            Ok(message) => {
                handle_message_from_vehicle(message, scripts.clone());
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
    scripts: watch::Sender<Vec<AutoScript>>,
) {
    match message {
        TheBeginnerCarOutgoingMessages::Log { log } => todo!(),
        TheBeginnerCarOutgoingMessages::SystemState {} => todo!(),
        TheBeginnerCarOutgoingMessages::WasmInstalled => todo!(),
        TheBeginnerCarOutgoingMessages::AutoScriptStarted {
            run_uuid,
            script_start_time_utc_ms,
        } => todo!(),
        TheBeginnerCarOutgoingMessages::AutoScriptStopped {
            run_uuid,
            script_stop_time_utc_ms,
            time_trials_dnf,
        } => todo!(),
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
