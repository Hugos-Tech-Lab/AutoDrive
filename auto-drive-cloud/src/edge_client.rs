use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::{
    RwLock,
    broadcast::{self, Sender},
    watch,
};
use tokio::time::sleep;
use tonic::transport::Channel;

use crate::auto_drive_edge_service::v1::{
    AutoScripts, GetVehiclesRequest, LogMessage,
    auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
};
use crate::edge_client::watch_auto_script_logs::watch_auto_script_logs;
use crate::edge_client::watch_scripts_overview::watch_scripts_overview;
pub mod watch_auto_script_logs;
pub mod watch_scripts_overview;
pub mod watch_vehicle_positions;

type HardwareHashMap = HashMap<String, HardwareDeviceState>;
type HardwarePositions = HashMap<String, Pose>;

pub enum StartAutoScriptResponse {
    Uuid { uuid: String },
    EdgeClientUnavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    pub number_plate: String,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

// TODO FILL THIS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McuStatus {
    cpu_perc: f32,
    // ...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub wifi_mcu: McuStatus,
    pub wasm_mcu: McuStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDeviceState {
    pub number_plate: String,
    pub category: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone)]
pub struct Logs {
    history: Arc<RwLock<Vec<LogMessage>>>,
    live_tx: Sender<LogMessage>,
}

// TODO: remove default here
impl Default for Logs {
    fn default() -> Self {
        let live_tx = broadcast::Sender::new(100);

        Self {
            history: Default::default(),
            live_tx,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vehicle {
    logs: Logs,
    position: watch::Sender<Option<Pose>>,
    status: watch::Sender<Option<Status>>,
}

#[derive(Debug, Clone)]
pub struct AutoScript {
    logs: Logs,
}

impl AutoScript {
    pub fn new(handle: AutoDriveEdgeServiceClient<Channel>, run_uuid: String) -> Self {
        let (logs, _) = watch_auto_script_logs(handle, run_uuid);
        Self { logs }
    }
}

#[derive(Debug, Clone)]
pub struct EdgeClient {
    pub handle: AutoDriveEdgeServiceClient<Channel>,
    pub scripts_overview: watch::Sender<AutoScripts>,
    pub vehicles: HashMap<String, Vehicle>,
    pub scripts: Arc<Mutex<HashMap<String, AutoScript>>>,
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

        let vehicles: HashMap<_, _> = client
            .get_vehicles(GetVehiclesRequest {})
            .await?
            .get_ref()
            .number_plates
            .iter()
            .map(|f| (f.clone(), Vehicle::default()))
            .collect();

        let (scripts_overview, _) = watch_scripts_overview(client.clone());

        Ok(EdgeClient {
            handle: client,
            scripts_overview,
            vehicles,
            scripts: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}
