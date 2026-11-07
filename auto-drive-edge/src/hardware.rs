use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DeviceCategory {
    TheBeginnerCar,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareMetadata {
    pub category: DeviceCategory,
    pub number_plate: String,
    pub ip: String,
    pub mac_address: String,
    pub aruco_marker_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub hardware: Vec<HardwareMetadata>,
}
