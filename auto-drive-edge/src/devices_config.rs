use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DeviceCategory {
    TheBeginnerCar,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DeviceId {
  TcA,
  TcB
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareMetadata {
    pub device_category: DeviceCategory,
    pub device_id: DeviceId,
    pub mdns_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceConfig {
    // pub devices: Vec<HardwareMetadata>,
    pub the_beginner_cars: Vec<HardwareMetadata>
}
