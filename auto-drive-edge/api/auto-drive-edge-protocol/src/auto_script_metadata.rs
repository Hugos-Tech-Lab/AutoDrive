use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    auto_script_lifecycle::AutoScriptLifecycle,
    auto_script_permission_level::AutoScriptPermissionLevel,
};

#[derive(Serialize, Deserialize, ts_rs::TS)]
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

//
