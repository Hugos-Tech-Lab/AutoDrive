use std::{error::Error, time::Duration};

use log::{info, warn};
use tokio::time::sleep;
use tonic::transport::Channel;

use crate::edge_client::{auto_scripts::AutoScripts, vehicle::Vehicle};
use auto_drive_edge_protocol::auto_drive_edge_service::v1::{
    GetVehiclesRequest, auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
};
pub mod auto_script;
pub mod auto_scripts;
pub mod grpc_connect;
pub mod logs;
pub mod vehicle;

#[derive(Debug, Clone)]
pub struct EdgeClient {
    pub handle: AutoDriveEdgeServiceClient<Channel>,
    pub vehicles: Vec<Vehicle>,
    pub scripts: AutoScripts,
}

impl EdgeClient {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let mut client = loop {
            #[cfg(feature = "development")]
            let auto_drive_edge_ip = "http://192.168.0.173:50051";
            #[cfg(feature = "production")]
            let auto_drive_edge_ip = "http://100.96.0.1:50051";

            match AutoDriveEdgeServiceClient::connect(auto_drive_edge_ip).await {
                Ok(c) => {
                    info!("Successfully connected to edge service.");
                    break c;
                }
                Err(err) => {
                    warn!("Failed to connect to edge service: {err}. Retrying in 5s...");
                    sleep(Duration::from_secs(5)).await;
                }
            }
        };

        let vehicles = client
            .get_vehicles(GetVehiclesRequest {})
            .await?
            .get_ref()
            .number_plates
            .iter()
            .map(|number_plate| Vehicle::new(client.clone(), number_plate.clone()))
            .collect();

        let scripts = AutoScripts::new(client.clone());

        Ok(EdgeClient {
            handle: client,
            vehicles,
            scripts,
        })
    }
}
