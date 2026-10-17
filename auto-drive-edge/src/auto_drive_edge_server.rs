use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::{
    auto_script_manager::AutoScriptManager, devices_config::DeviceConfig,
    the_beginner_car_comms::TheBeginnerCarComms,
    vehicle_hardware_telemetry::VehicleHardwareTelemetry,
};

pub struct AutoDriveEdgeServer {
    pub the_beginner_cars: Vec<TheBeginnerCarComms>,
    pub the_beginner_car_handles: Vec<JoinHandle<()>>,
    pub telemetry: VehicleHardwareTelemetry,
    pub auto_script_manager: Arc<AutoScriptManager>,
}

impl AutoDriveEdgeServer {
    pub fn new(config: DeviceConfig) -> Self {
        let (the_beginner_cars, the_beginner_car_handles): (
            Vec<TheBeginnerCarComms>,
            Vec<JoinHandle<()>>,
        ) = config
            .the_beginner_cars
            .iter()
            .map(|d| TheBeginnerCarComms::new(d.clone()))
            .unzip();

        let telemetry = VehicleHardwareTelemetry::new(the_beginner_cars.clone());

        let auto_script_manager = AutoScriptManager::new(the_beginner_cars.clone());

        Self {
            the_beginner_cars,
            the_beginner_car_handles,
            telemetry,
            auto_script_manager: auto_script_manager.into(),
        }
    }
}
