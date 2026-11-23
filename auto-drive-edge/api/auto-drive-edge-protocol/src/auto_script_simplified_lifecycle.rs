use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum AutoScriptSimplifiedLifecycle {
    InTheQueue,
    Driving,
    Stopped,
}
