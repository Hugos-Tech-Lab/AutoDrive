use serde::{Deserialize, Serialize};

use crate::{
    auto_drive_edge_service::v1, auto_script_simplified_lifecycle::AutoScriptSimplifiedLifecycle,
};

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

impl AutoScriptLifecycle {
    pub fn simplified(&self) -> AutoScriptSimplifiedLifecycle {
        match self {
            // IN THE QUEUE
            AutoScriptLifecycle::Idle => AutoScriptSimplifiedLifecycle::InTheQueue,
            // DRIVING
            AutoScriptLifecycle::SendingToWifiMcu
            | AutoScriptLifecycle::ReceivedByWifiMcu
            | AutoScriptLifecycle::SendingToWasmMcu
            | AutoScriptLifecycle::ReceivedByWasmMcu
            | AutoScriptLifecycle::StartedCallingMain
            | AutoScriptLifecycle::Cancelling => AutoScriptSimplifiedLifecycle::Driving,
            // STOPPED
            AutoScriptLifecycle::StoppedByScriptEnd
            | AutoScriptLifecycle::StoppedBySafety
            | AutoScriptLifecycle::StoppedByHttpCancel
            | AutoScriptLifecycle::StoppedByDisconnect => AutoScriptSimplifiedLifecycle::Stopped,
            // END
        }
    }
}

impl From<AutoScriptLifecycle> for v1::AutoScriptLifecycle {
    fn from(value: AutoScriptLifecycle) -> Self {
        match value {
            AutoScriptLifecycle::Idle => v1::AutoScriptLifecycle::Idle,
            AutoScriptLifecycle::SendingToWifiMcu => v1::AutoScriptLifecycle::SendingToWifiMcu,
            AutoScriptLifecycle::ReceivedByWifiMcu => v1::AutoScriptLifecycle::ReceivedByWifiMcu,
            AutoScriptLifecycle::SendingToWasmMcu => v1::AutoScriptLifecycle::SendingToWasmMcu,
            AutoScriptLifecycle::ReceivedByWasmMcu => v1::AutoScriptLifecycle::ReceivedByWasmMcu,
            AutoScriptLifecycle::StartedCallingMain => v1::AutoScriptLifecycle::StartedCallingMain,
            AutoScriptLifecycle::Cancelling => v1::AutoScriptLifecycle::Cancelling,
            AutoScriptLifecycle::StoppedByScriptEnd => v1::AutoScriptLifecycle::StoppedByScriptEnd,
            AutoScriptLifecycle::StoppedBySafety => v1::AutoScriptLifecycle::StoppedBySafety,
            AutoScriptLifecycle::StoppedByHttpCancel => {
                v1::AutoScriptLifecycle::StoppedByHttpCancel
            }
            AutoScriptLifecycle::StoppedByDisconnect => {
                v1::AutoScriptLifecycle::StoppedByDisconnect
            }
        }
    }
}

impl TryFrom<v1::AutoScriptLifecycle> for AutoScriptLifecycle {
    type Error = ();

    fn try_from(value: v1::AutoScriptLifecycle) -> Result<Self, Self::Error> {
        match value {
            v1::AutoScriptLifecycle::Unspecified => Err(()),
            v1::AutoScriptLifecycle::Idle => Ok(AutoScriptLifecycle::Idle),
            v1::AutoScriptLifecycle::SendingToWifiMcu => Ok(AutoScriptLifecycle::SendingToWifiMcu),
            v1::AutoScriptLifecycle::ReceivedByWifiMcu => {
                Ok(AutoScriptLifecycle::ReceivedByWifiMcu)
            }
            v1::AutoScriptLifecycle::SendingToWasmMcu => Ok(AutoScriptLifecycle::SendingToWasmMcu),
            v1::AutoScriptLifecycle::ReceivedByWasmMcu => {
                Ok(AutoScriptLifecycle::ReceivedByWasmMcu)
            }
            v1::AutoScriptLifecycle::StartedCallingMain => {
                Ok(AutoScriptLifecycle::StartedCallingMain)
            }
            v1::AutoScriptLifecycle::Cancelling => Ok(AutoScriptLifecycle::Cancelling),
            v1::AutoScriptLifecycle::StoppedByScriptEnd => {
                Ok(AutoScriptLifecycle::StoppedByScriptEnd)
            }
            v1::AutoScriptLifecycle::StoppedBySafety => Ok(AutoScriptLifecycle::StoppedBySafety),
            v1::AutoScriptLifecycle::StoppedByHttpCancel => {
                Ok(AutoScriptLifecycle::StoppedByHttpCancel)
            }
            v1::AutoScriptLifecycle::StoppedByDisconnect => {
                Ok(AutoScriptLifecycle::StoppedByDisconnect)
            }
        }
    }
}
