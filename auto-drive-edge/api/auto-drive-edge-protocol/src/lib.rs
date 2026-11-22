#[derive(Clone, Debug)]
pub enum AutoScriptType {
    TimeTrial,
    Reposition,
    ConfigAdjust,
}

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
