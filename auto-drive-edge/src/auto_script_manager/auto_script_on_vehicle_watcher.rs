
use chrono::{DateTime, Utc};
use log::{info, warn};
use the_beginner_car_tcp_protocol::TheBeginnerCarOutgoingMessages;
use tokio::{
    sync::{
        broadcast::{self},
        watch,
    },
    task::JoinHandle,
};

use crate::{
    auto_script_manager::auto_script::AutoScript, the_beginner_car_comms::TheBeginnerCarComms,
};

pub struct AutoScriptOnVehicleWatcher {
    watchers: Vec<JoinHandle<()>>,
}

impl AutoScriptOnVehicleWatcher {
    pub fn new(
        the_beginner_cars: Vec<TheBeginnerCarComms>,
        scripts: watch::Sender<Vec<AutoScript>>,
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
        scripts: watch::Sender<Vec<AutoScript>>,
    ) {
        let mut incoming_rx = the_beginner_car.incoming_tx.subscribe();

        loop {
            match incoming_rx.recv().await {
                Ok(message) => match message {
                    TheBeginnerCarOutgoingMessages::AutoScriptStarted {
                        run_uuid,
                        script_start_time_utc_ms,
                    } => {
                        //
                    }
                    TheBeginnerCarOutgoingMessages::AutoScriptStopped {
                        run_uuid,
                        script_stop_time_utc_ms,
                        time_trials_dnf,
                    } => {
                        Self::on_hardware_response_auto_script_stopped(
                            scripts.clone(),
                            &the_beginner_car,
                            run_uuid,
                            script_stop_time_utc_ms,
                            time_trials_dnf,
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
        auto_scripts: watch::Sender<Vec<AutoScript>>,
        vehicle: &TheBeginnerCarComms,
        run_uuid: String,
        script_stop_time_utc_ms: u64,
        time_trials_dnf: bool,
    ) {
        auto_scripts.send_modify(|auto_scripts| {
            let auto_script = auto_scripts
                .iter_mut()
                .find(|auto_script| auto_script.run_uuid == *run_uuid);

            let Some(auto_script) = auto_script else {
                warn!("AutoScript not found ({run_uuid})");
                return;
            };

            if auto_script.script_stop_time.is_some() {
                warn!("Hardware stopped but script already has a stop time.. we stopped twice? ({run_uuid})");
                return;
            }

            auto_script.hardware.get_or_insert(vehicle.clone());

            let script_stop_time = DateTime::<Utc>::from_timestamp_millis(script_stop_time_utc_ms as i64);

            let Some(script_stop_time) = script_stop_time else {
                warn!("Script stop time ({script_stop_time_utc_ms}) invalid");
                return;
            };

            auto_script.script_stop_time = Some(script_stop_time);

            if auto_script.time_trials_dnf.is_some() {
                warn!("Hardware stopped and DNF is some? ({run_uuid})");
                return;  
            } 

            auto_script.time_trials_dnf = Some(time_trials_dnf);
        });
    }
}
