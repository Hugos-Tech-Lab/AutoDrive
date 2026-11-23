use auto_drive_edge_protocol::auto_drive_edge_service::v1::auto_drive_edge_service_client::AutoDriveEdgeServiceClient;
use tokio::sync::watch;
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

#[derive(Debug, Clone)]
pub struct AutoScript {
    pub run_uuid: String,
    pub logs: Logs,
    pub status: watch::Sender<AutoScriptStatus>,
}

impl AutoScript {
    pub fn new(handle: AutoDriveEdgeServiceClient<Channel>, run_uuid: String) -> Self {
        let logs = Logs::from_auto_script(handle, run_uuid.clone());
        let status = watch::Sender::<AutoScriptStatus>::new(AutoScriptStatus::ReceivedByHardware); // TODO send messages to this
        Self {
            run_uuid,
            logs,
            status,
        }
    }
}
