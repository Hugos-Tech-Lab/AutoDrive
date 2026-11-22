use hardware_config::HardwareConfig;

use crate::{
    auto_script_manager::AutoScripts, vehicle::the_beginner_car::TheBeginnerCar,
    vehicle_hardware_telemetry::VehicleHardwareTelemetry,
};

pub struct AutoDriveEdgeServer {
    pub auto_scripts: AutoScripts,
    pub vehicles: Vec<TheBeginnerCar>,
    pub telemetry: VehicleHardwareTelemetry,
}

impl AutoDriveEdgeServer {
    pub fn new(config: HardwareConfig) -> Self {
        let auto_scripts = AutoScripts::new();

        let vehicles: Vec<TheBeginnerCar> = config
            .the_beginner_cars
            .iter()
            .map(|d| TheBeginnerCar::new(d.clone(), auto_scripts.clone()))
            .collect();

        let telemetry = VehicleHardwareTelemetry::new(vehicles.clone());

        Self {
            auto_scripts,
            vehicles,
            telemetry,
        }
    }
}
