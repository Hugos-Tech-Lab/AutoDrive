use std::sync::Arc;

use auto_drive_edge_protocol::auto_drive_edge_service::v1::{
    WatchHardwarePoseRequest, auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
};
use log::{info, warn};
use tokio::{sync::watch, task::JoinHandle};
use tonic::transport::Channel;

use crate::edge_client::{grpc_connect::grpc_connect, logs::Logs};

// // TODO FILL THIS
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct McuStatus {
//     cpu_perc: f32,
//     // ...
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct VehicleHardwareStatus {
//     pub wifi_mcu: McuStatus,
//     pub wasm_mcu: McuStatus,
// }

#[derive(Debug, Clone)]
pub struct VehiclePosition {
    pub number_plate: String,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub number_plate: String,
    pub logs: Logs,
    vehicle_position: watch::Receiver<Option<VehiclePosition>>,
    watch_vehicle_position_task: Arc<JoinHandle<()>>,
    // status: watch::Receiver<Option<Status>>,
}

impl Vehicle {
    pub fn new(client: AutoDriveEdgeServiceClient<Channel>, number_plate: String) -> Self {
        let logs = Logs::from_vehicle(client.clone(), number_plate.clone());

        let (vehicle_position_tx, vehicle_position_rx) =
            watch::channel::<Option<VehiclePosition>>(None);

        let watch_vehicle_position_task = tokio::spawn(watch_vehicle_position(
            client.clone(),
            vehicle_position_tx,
            number_plate.clone(),
        ));

        Self {
            number_plate,
            logs,
            vehicle_position: vehicle_position_rx,
            watch_vehicle_position_task: Arc::new(watch_vehicle_position_task),
        }
    }
}

async fn watch_vehicle_position(
    client: AutoDriveEdgeServiceClient<Channel>,
    vehicle_position: watch::Sender<Option<VehiclePosition>>,
    number_plate: String,
) {
    let mut stream = grpc_connect(
        client.clone(),
        WatchHardwarePoseRequest {
            number_plate: number_plate.clone(),
        },
        AutoDriveEdgeServiceClient::watch_hardware_pose,
    )
    .await;

    loop {
        let response = stream.message().await;
        let response = match response {
            Ok(Some(response)) => response,
            Err(err) => {
                warn!("error from the edge: {}", err);
                continue;
            }
            Ok(None) => break,
        };

        vehicle_position.send_modify(|vehicle_position| {
            *vehicle_position = Some(VehiclePosition {
                number_plate: response.number_plate,
                x: response.x,
                y: response.y,
                rotation: response.rotation_radians,
            })
        });
    }

    info!("no longer watching vehicle ({number_plate}) position");
}
