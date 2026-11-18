use std::sync::Arc;

use auto_drive_common_types::{LogLevel, LogMessage, LogSystemOrigin};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use tokio::sync::{
    RwLock,
    broadcast::{self, Sender},
};
use uuid::Uuid;

use crate::{
    auto_script_manager::{AutoScriptEvents, AutoScriptStatus},
    the_beginner_car_comms::TheBeginnerCarComms,
};

#[derive(Clone, Debug)]
pub enum AutoScriptType {
    TimeTrial,
    Reposition,
    ConfigAdjust,
}

// pub enum UserType {
//     Regular,
//     Privileged,
// }

#[derive(Clone, Debug)]
pub struct AutoScript {
    pub owner_name: String,
    pub owner_comment: String,
    pub run_uuid: String,
    pub run_number: u32,
    pub uploaded_script: Arc<[u8]>,
    pub uploaded_script_hash: String,
    // pub long_lived_hardware_listener_handle: Arc<JoinHandle<()>>,
    pub logs_live_tx: Sender<LogMessage>,
    pub logs_history: Arc<RwLock<Vec<LogMessage>>>,
    pub hardware: Option<TheBeginnerCarComms>,
    pub hardware_response: AutoScriptStatus,
    pub script_queue_time: DateTime<Utc>,
    pub script_start_time: Option<DateTime<Utc>>,
    pub vehicle_start_time: Option<DateTime<Utc>>,
    pub vehicle_finish_time: Option<DateTime<Utc>>,
    pub script_stop_time: Option<DateTime<Utc>>,
    pub script_type: AutoScriptType,
    // pub user_type: U
}

impl AutoScript {
    pub fn new(
        owner_name: String,
        owner_comment: String,
        run_number: u32,
        uploaded_script: Arc<[u8]>,
        script_type: AutoScriptType,
    ) -> Self {
        let run_uuid = Uuid::new_v4().to_string();

        let uploaded_script_hash = Sha256::digest(&uploaded_script);
        let uploaded_script_hash = hex::encode(Sha256::digest(uploaded_script_hash));

        let (logs_live_tx, _) = broadcast::channel(100);
        let logs_history = Arc::new(RwLock::new(Vec::new()));

        // TODO: bring this back
        // let long_lived_hardware_listener_handle = tokio::task::spawn({
        //     let hardware_rx = hardware.incoming_tx.subscribe();
        //     let run_uuid = run_uuid.clone();
        //     let events_history = events_history.clone();
        //     let events_tx = events_tx.clone();

        //     async move {
        //         AutoScript::long_lived_hardware_listener(
        //             hardware_rx,
        //             events_history,
        //             events_tx,
        //             run_uuid,
        //         )
        //         .await
        //     }
        // });

        Self {
            owner_name,
            owner_comment,
            run_uuid,
            run_number,
            uploaded_script,
            uploaded_script_hash,
            // long_lived_hardware_listener_handle: Arc::from(long_lived_hardware_listener_handle),
            logs_live_tx,
            logs_history,
            hardware: None,
            hardware_response: AutoScriptStatus::NotReceivedByHardware,
            script_queue_time: Utc::now(),
            script_start_time: None,
            script_stop_time: None,
            vehicle_start_time: None,
            vehicle_finish_time: None,
            script_type,
        }
    }

    // pub async fn long_lived_hardware_listener(
    //     mut hardware_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
    //     events_history: Arc<RwLock<Vec<AutoScriptEvents>>>,
    //     events_tx: broadcast::Sender<AutoScriptEvents>,
    //     run_uuid: String,
    // ) {
    //     loop {
    //         // TODO: add `tokio::select!` here later to handle the cancellation token
    //         match hardware_rx.recv().await {
    //             Ok(message) => match message {
    //                 TheBeginnerCarOutgoingMessages::Log { log } => {
    //                     // if log.trace_id.as_deref() == Some(&run_uuid) {
    //                     let event = AutoScriptEvents::LogMessage {
    //                         log_message: log.clone(),
    //                     };
    //                     let mut hist_lock = events_history.write().await;
    //                     hist_lock.push(event.clone());
    //                     let _ = events_tx.send(event);
    //                     // }
    //                 }
    //                 TheBeginnerCarOutgoingMessages::AutoScriptStopped => {
    //                     let event = AutoScriptEvents::ScriptExited;
    //                     let mut hist_lock = events_history.write().await;
    //                     hist_lock.push(event.clone());
    //                     let _ = events_tx.send(event);
    //                 }
    //                 _ => {}
    //             },
    //             Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
    //                 eprintln!("Car watcher lagged; skipped {skipped} messages");
    //             }
    //             Err(tokio::sync::broadcast::error::RecvError::Closed) => {
    //                 info!("Closed");
    //                 // The car disconnected or channel closed, exit the loop
    //                 break;
    //             }
    //         };
    //     }
    // }

    pub async fn send_log_info(&self, message: String) {
        let event = AutoScriptEvents::LogMessage {
            log_message: LogMessage {
                system_origin: LogSystemOrigin::WasmMcu, // TODO: change this origin to Edge.
                level: LogLevel::Info,                   // TODO: support for other than info.
                target: "".to_string(),                  // TODO: what does target mean?.
                message: message,
                trace_id: Some(self.run_uuid.clone()),
            },
        };
        let mut hist_lock = self.events_history.write().await;
        hist_lock.push(event.clone());
        let _ = self.events_tx.send(event);
    }
}
