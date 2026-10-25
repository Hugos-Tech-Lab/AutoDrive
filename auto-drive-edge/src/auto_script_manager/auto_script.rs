use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::{RwLock, broadcast::Sender};

use crate::{
    auto_script_manager::{AutoScriptEvents, AutoScriptResponseFromHardware},
    the_beginner_car_comms::TheBeginnerCarComms,
};

// TODO: separate out metadata from content so cloning is cheaper....
#[derive(Clone, Debug)]
pub struct AutoScript {
    pub owner_name: String,
    pub run_uuid: String,
    pub upload_file_hash: String,
    pub run_number: u32,
    pub script: Vec<u8>,
    pub events: Sender<AutoScriptEvents>,
    pub history: Arc<RwLock<Vec<AutoScriptEvents>>>,
    pub device: TheBeginnerCarComms,
    pub hardware_response: AutoScriptResponseFromHardware,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
}
