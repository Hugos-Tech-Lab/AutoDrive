use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::{sync::watch, task::JoinHandle};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehiclePositionInternal {
    pub number_plate: String,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone)]
pub struct VehiclePosition {
    vehicle_position: watch::Receiver<VehiclePositionInternal>,
    watch_vehicle_position_task: Arc<JoinHandle<()>>,
}
