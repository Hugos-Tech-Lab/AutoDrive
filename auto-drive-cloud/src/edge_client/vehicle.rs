use tonic::transport::Channel;

use crate::{
    auto_drive_edge_service::v1::auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
    edge_client::logs::Logs,
};

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub number_plate: String,
    pub logs: Logs,
    // position: watch::Receiver<Option<Pose>>,
    // status: watch::Receiver<Option<Status>>,
}

impl Vehicle {
    pub fn new(client: AutoDriveEdgeServiceClient<Channel>, number_plate: String) -> Self {
        let logs = Logs::from_vehicle(client.clone(), number_plate.clone());

        Self {
            number_plate,
            logs,
            // position: (),
            // status: (),
        }
    }
}
