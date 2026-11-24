use auto_drive_edge_protocol::{
    auto_drive_edge_service::v1::auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
    auto_script_metadata::AutoScriptMetadata,
};
use tonic::transport::Channel;

use crate::edge_client::logs::Logs;

#[derive(Clone, Debug)]
pub enum AutoScriptStatus {
    NotReceivedByHardware,
    ReceivedByHardware,
    CallingMain,
    StoppedBySelf,
    CancelledBySafety,
    CancelledManually,
}

#[derive(Clone, Debug)]
pub struct AutoScript {
    pub metadata: AutoScriptMetadata,
    pub logs: Logs,
}

impl AutoScript {
    pub fn new(handle: AutoDriveEdgeServiceClient<Channel>, metadata: AutoScriptMetadata) -> Self {
        let logs = Logs::from_auto_script(handle, metadata.run_uuid.clone());
        Self { metadata, logs }
    }
}
