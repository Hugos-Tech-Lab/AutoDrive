use hardware_config::HardwareConfig;

use crate::{
    auto_script_manager::{AutoScriptManager, AutoScripts},
    vehicle::the_beginner_car::TheBeginnerCar,
    vehicle_hardware_telemetry::VehicleHardwareTelemetry,
};

pub struct AutoDriveEdgeServer {
    pub auto_scripts: AutoScripts,
    pub the_beginner_cars: Vec<TheBeginnerCar>,
    pub telemetry: VehicleHardwareTelemetry,
}

impl AutoDriveEdgeServer {
    pub fn new(config: HardwareConfig) -> Self {
        let auto_scripts = AutoScripts::new();
        //
        //
        let the_beginner_cars = config
            .the_beginner_cars
            .iter()
            .map(|d| TheBeginnerCar::new(d.clone()))
            .collect();

        let telemetry = VehicleHardwareTelemetry::new(the_beginner_cars.clone());

        let auto_script_manager = AutoScriptManager::new(the_beginner_cars.clone());

        Self {
            the_beginner_cars,
            telemetry,
            auto_script_manager: auto_script_manager.into(),
        }
    }
}
