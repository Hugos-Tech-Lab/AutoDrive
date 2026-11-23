use serde::{Deserialize, Serialize};

use crate::auto_drive_edge_service::v1;

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum AutoScriptLifecycle {
    Idle,
    SendingToWifiMcu,
    ReceivedByWifiMcu,
    SendingToWasmMcu,
    ReceivedByWasmMcu,
    StartedCallingMain,
    Cancelling,
    StoppedByScriptEnd,
    StoppedBySafety,
    StoppedByHttpCancel,
    StoppedByDisconnect,
}

impl From<AutoScriptLifecycle> for v1::AutoScriptLifecycle {
    fn from(value: AutoScriptLifecycle) -> Self {
        match value {
            AutoScriptLifecycle::Idle => v1::AutoScriptLifecycle::Queued,
            AutoScriptLifecycle::SendingToWifiMcu => todo!(),
            AutoScriptLifecycle::ReceivedByWifiMcu => todo!(),
            AutoScriptLifecycle::SendingToWasmMcu => todo!(),
            AutoScriptLifecycle::ReceivedByWasmMcu => todo!(),
            AutoScriptLifecycle::StartedCallingMain => todo!(),
            AutoScriptLifecycle::Cancelling => todo!(),
            AutoScriptLifecycle::StoppedByScriptEnd => todo!(),
            AutoScriptLifecycle::StoppedBySafety => todo!(),
            AutoScriptLifecycle::StoppedByHttpCancel => todo!(),
            AutoScriptLifecycle::StoppedByDisconnect => todo!(),
        }
    }
}

impl From<v1::AutoScriptLifecycle> for AutoScriptLifecycle {
    fn from(value: v1::AutoScriptLifecycle) -> Self {
        Self {}
    }
}
