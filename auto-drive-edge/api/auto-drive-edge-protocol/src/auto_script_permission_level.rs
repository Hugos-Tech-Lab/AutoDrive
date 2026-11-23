use serde::{Deserialize, Serialize};

use crate::auto_drive_edge_service::v1;

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum AutoScriptPermissionLevel {
    Standard,
    Elevated,
}

impl From<AutoScriptPermissionLevel> for v1::AutoScriptPermissionLevel {
    fn from(value: AutoScriptPermissionLevel) -> Self {
        match value {
            AutoScriptPermissionLevel::Standard => {
                v1::AutoScriptPermissionLevel::AutoScriptTypeStandard
            }
            AutoScriptPermissionLevel::Elevated => {
                v1::AutoScriptPermissionLevel::AutoScriptTypeElevated
            }
        }
    }
}

impl TryFrom<v1::AutoScriptPermissionLevel> for AutoScriptPermissionLevel {
    type Error = ();

    fn try_from(value: v1::AutoScriptPermissionLevel) -> Result<Self, Self::Error> {
        match value {
            v1::AutoScriptPermissionLevel::AutoScriptTypeUnspecified => Err(()),
        }
    }
}
