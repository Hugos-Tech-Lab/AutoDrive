use std::sync::Arc;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{auto_scripts::AutoScriptLifecycle, logs::Logs};

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
    pub logs: Logs,
    pub vehicle_number_plate: Option<String>,
    pub lifecycle: AutoScriptLifecycle,
    pub script_queue_time: DateTime<Utc>,
    pub script_start_time: Option<DateTime<Utc>>,
    pub time_trials_start_time: Option<DateTime<Utc>>,
    pub time_trials_finish_time: Option<DateTime<Utc>>,
    pub time_trials_dnf: Option<bool>,
    pub script_stop_time: Option<DateTime<Utc>>,
    pub script_type: AutoScriptType,
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

        Self {
            owner_name,
            owner_comment,
            run_uuid,
            run_number,
            uploaded_script,
            uploaded_script_hash,
            logs: Logs::new(),
            vehicle_number_plate: None,
            lifecycle: AutoScriptLifecycle::Idle,
            script_queue_time: Utc::now(),
            script_start_time: None,
            script_stop_time: None,
            time_trials_start_time: None,
            time_trials_finish_time: None,
            time_trials_dnf: None,
            script_type,
        }
    }

    // pub async fn send_log_info(&self, message: String) {
    //     let event = AutoScriptEvents::LogMessage {
    //         log_message: LogMessage {
    //             system_origin: LogSystemOrigin::WasmMcu, // TODO: change this origin to Edge.
    //             level: LogLevel::Info,                   // TODO: support for other than info.
    //             target: "".to_string(),                  // TODO: what does target mean?.
    //             message: message,
    //             trace_id: Some(self.run_uuid.clone()),
    //         },
    //     };
    //     let mut hist_lock = self.events_history.write().await;
    //     hist_lock.push(event.clone());
    //     let _ = self.events_tx.send(event);
    // }
}
