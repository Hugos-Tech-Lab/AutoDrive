use std::sync::Arc;

use crate::{
    devices_config::DeviceConfig, physical_hardware::PhysicalHardware,
    vehicle_hardware_telemetry::VehicleHardwareTelemetry,
};

pub struct AutoDriveEdgeServer {
    pub hardware: Arc<PhysicalHardware>,
    pub telemetry: VehicleHardwareTelemetry,
    // pub http_server: Htt
}

impl AutoDriveEdgeServer {
    pub fn new(config: DeviceConfig) -> Self {
        // let the_beginner_cars = TheBeginnerCars::new(the_beginner_cars);
        let hardware = Arc::new(PhysicalHardware::new(config));
        let telemetry = VehicleHardwareTelemetry::new(hardware.clone());

        Self {
            hardware,
            telemetry,
        }
    }
}
