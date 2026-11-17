use std::sync::Arc;

use log::{info, warn};
use the_beginner_car_tcp_protocol::TheBeginnerCarOutgoingMessages;
use tokio::{
    sync::{
        Mutex,
        broadcast::{self, Sender},
        watch,
    },
    task::JoinHandle,
};

use crate::{
    auto_script_manager::{AutoScripts, auto_script::AutoScript},
    the_beginner_car_comms::TheBeginnerCarComms,
};

pub struct AutoScriptOnVehicleWatcher {
    watchers: Vec<JoinHandle<()>>,
}

impl AutoScriptOnVehicleWatcher {
    pub fn new(
        the_beginner_cars: Vec<TheBeginnerCarComms>,
        scripts: watch::Sender<AutoScripts>,
    ) -> Self {
        let watchers = the_beginner_cars
            .into_iter()
            .map(|the_beginner_car| {
                tokio::spawn({
                    let scripts = scripts.clone();
                    async {
                        Self::watch_vehicle_state_change(the_beginner_car, scripts).await;
                    }
                })
            })
            .collect();
        Self { watchers }
    }

    async fn watch_vehicle_state_change(
        the_beginner_car: TheBeginnerCarComms,
        scripts: watch::Sender<AutoScripts>,
    ) {
        let mut incoming_rx = the_beginner_car.incoming_tx.subscribe();

        loop {
            match incoming_rx.recv().await {
                Ok(message) => match message {
                    TheBeginnerCarOutgoingMessages::AutoScriptRunning {
                        run_uuid,
                        script_start_time_utc_ms,
                    } => {
                        //
                    }
                    TheBeginnerCarOutgoingMessages::AutoScriptStopped {
                        run_uuid,
                        script_stop_time_utc_ms,
                        vehicle_start_time_utc_ms,
                        vehicle_finish_time_utc_ms,
                    } => {
                        Self::on_hardware_response_auto_script_stopped(
                            scripts.clone(),
                            the_beginner_car.metadata.number_plate.to_string(),
                            run_uuid,
                            script_stop_time_utc_ms,
                            vehicle_start_time_utc_ms,
                            vehicle_finish_time_utc_ms,
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

    // TODO: fill / use these fields
    pub async fn on_hardware_response_auto_script_stopped(
        scripts: watch::Sender<AutoScripts>,
        number_plate: String,
        run_uuid: String,
        script_stop_time_utc_ms: u64,
        vehicle_start_time_utc_ms: u64,
        vehicle_finish_time_utc_ms: u64,
    ) {
        let mut stopped = Vec::new();

        scripts.send_modify(|state| {
            stopped = state
                .running
                .extract_if(.., |auto_script| auto_script.run_uuid == run_uuid)
                .collect();

            if stopped.len() != 1 {
                warn!("Unexpected number of scripts removed: ({})", stopped.len());
            }

            state.stopped.extend(stopped.iter().cloned());
        });

        for stopped_script in stopped {
            stopped_script
                .send_log_info("Script exited".to_string())
                .await;
        }
    }
}
