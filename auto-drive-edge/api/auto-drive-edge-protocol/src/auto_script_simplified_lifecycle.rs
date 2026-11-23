use crate::auto_drive_edge_service::v1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum AutoScriptSimplifiedLifecycle {
    InTheQueue,
    Driving,
    Stopped,
}

impl From<AutoScriptSimplifiedLifecycle> for v1::AutoScriptSimplifiedLifecycle {
    fn from(value: AutoScriptSimplifiedLifecycle) -> Self {
        match value {
            AutoScriptSimplifiedLifecycle::InTheQueue => {
                v1::AutoScriptSimplifiedLifecycle::InTheQueue
            }
            AutoScriptSimplifiedLifecycle::Driving => v1::AutoScriptSimplifiedLifecycle::Driving,
            AutoScriptSimplifiedLifecycle::Stopped => v1::AutoScriptSimplifiedLifecycle::Stopped,
        }
    }
}

impl TryFrom<v1::AutoScriptSimplifiedLifecycle> for AutoScriptSimplifiedLifecycle {
    type Error = ();

    fn try_from(value: v1::AutoScriptSimplifiedLifecycle) -> Result<Self, Self::Error> {
        match value {
            v1::AutoScriptSimplifiedLifecycle::Unspecified => Err(()),
            v1::AutoScriptSimplifiedLifecycle::InTheQueue => Ok(Self::InTheQueue),
            v1::AutoScriptSimplifiedLifecycle::Driving => Ok(Self::Driving),
            v1::AutoScriptSimplifiedLifecycle::Stopped => Ok(Self::Stopped),
        }
    }
}
