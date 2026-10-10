use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DeviceType {
    TheBeginnerCar,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DeviceBuild {
  BuildA,
  BuildB,
  BuildC
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceMetadata {
    pub device_type: DeviceType,
    pub build: DeviceBuild,
    pub mdns_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub devices: Vec<DeviceMetadata>,
}
