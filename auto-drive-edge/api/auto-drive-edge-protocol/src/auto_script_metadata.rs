use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    auto_drive_edge_service::v1, auto_script_lifecycle::AutoScriptLifecycle,
    auto_script_permission_level::AutoScriptPermissionLevel,
};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, PartialEq)]
#[ts(export)]
pub struct AutoScriptMetadata {
    pub owner_name: String,
    pub owner_comment: String,
    pub run_uuid: String,
    pub run_number: u32,
    pub uploaded_script_hash: String,
    pub vehicle_number_plate: Option<String>,
    pub lifecycle: AutoScriptLifecycle,
    pub script_queue_time: DateTime<Utc>,
    pub script_start_time: Option<DateTime<Utc>>,
    pub time_trials_start_time: Option<DateTime<Utc>>,
    pub time_trials_finish_time: Option<DateTime<Utc>>,
    pub time_trials_dnf: Option<bool>,
    pub script_stop_time: Option<DateTime<Utc>>,
    pub permission_level: AutoScriptPermissionLevel,
}

impl From<AutoScriptMetadata> for v1::AutoScriptMetadata {
    fn from(value: AutoScriptMetadata) -> Self {
        Self {
            owner_name: value.owner_name,
            owner_comment: value.owner_comment,
            uploaded_script_hash: value.uploaded_script_hash,
            script_queue_time: value.script_queue_time.to_rfc3339(),
            script_start_time: value.script_start_time.map(|v| v.to_rfc3339()),
            vehicle_start_time: value.time_trials_start_time.map(|v| v.to_rfc3339()),
            vehicle_finish_time: value.time_trials_finish_time.map(|v| v.to_rfc3339()),
            script_stop_time: value.script_stop_time.map(|v| v.to_rfc3339()),
            number_plate: value.vehicle_number_plate,
            run_number: value.run_number,
            run_uuid: value.run_uuid,
            permission_level: v1::AutoScriptPermissionLevel::from(value.permission_level).into(),
            lifecycle: v1::AutoScriptLifecycle::from(value.lifecycle.clone()).into(),
            simplified_lifecycle: v1::AutoScriptSimplifiedLifecycle::from(
                value.lifecycle.simplified(),
            )
            .into(),
        }
    }
}
