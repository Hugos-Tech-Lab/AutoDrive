use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auto_script_type::AutoScriptType;

pub mod auto_drive_edge_service {
    pub mod v1 {
        tonic::include_proto!("auto_drive_edge_service.v1");
    }
}

pub mod auto_script_lifecycle;
pub mod auto_script_metadata;
pub mod auto_script_simplified_lifecycle;
pub mod auto_script_type;
