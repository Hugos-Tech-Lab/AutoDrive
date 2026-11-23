use std::sync::Arc;

use auto_drive_edge_protocol::{
    auto_script_lifecycle::AutoScriptLifecycle, auto_script_metadata::AutoScriptMetadata,
    auto_script_permission_level::AutoScriptPermissionLevel,
};
use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::logs::Logs;

#[derive(Clone, Debug)]
pub struct AutoScriptPayload {
    pub uploaded_script: Arc<[u8]>,
    pub logs: Logs,
}

#[derive(Clone, Debug)]
pub struct AutoScript {
    pub metadata: AutoScriptMetadata,
    pub payload: AutoScriptPayload,
}

impl AutoScript {
    pub fn new(
        owner_name: String,
        owner_comment: String,
        run_number: u32,
        uploaded_script: Arc<[u8]>,
        permission_level: AutoScriptPermissionLevel,
    ) -> Self {
        let run_uuid = Uuid::new_v4().to_string();

        let uploaded_script_hash = Sha256::digest(&uploaded_script);
        let uploaded_script_hash = hex::encode(Sha256::digest(uploaded_script_hash));

        Self {
            metadata: AutoScriptMetadata {
                owner_name,
                owner_comment,
                run_uuid,
                run_number,
                uploaded_script_hash,
                vehicle_number_plate: None,
                lifecycle: AutoScriptLifecycle::Idle,
                script_queue_time: Utc::now(),
                script_start_time: None,
                time_trials_start_time: None,
                time_trials_finish_time: None,
                time_trials_dnf: None,
                script_stop_time: None,
                permission_level,
            },
            payload: AutoScriptPayload {
                uploaded_script,
                logs: Logs::new(),
            },
        }
    }
}
