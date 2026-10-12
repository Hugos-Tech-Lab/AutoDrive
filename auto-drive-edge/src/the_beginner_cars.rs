use opentelemetry::{global, metrics::Gauge};

use crate::{devices_config::{DeviceBuild, DeviceMetadata}, the_beginner_car::TheBeginnerCar};

#[derive(Clone)]
pub struct TheBeginnerCarVehicleTelemetry {
    // motor_a_set_speed: Gauge<f64>,
    // motor_b_set_speed: Gauge<f64>,
    pub motor_battery_voltage: Gauge<f64>,
    pub mcu_battery_voltage: Gauge<f64>,
}

impl TheBeginnerCarVehicleTelemetry {
    pub fn new() -> Self {
        Self {
            motor_battery_voltage,
            mcu_battery_voltage
        }
    }
}

pub struct AutoScript {
    pub data: Vec<u8>
}

pub struct TheBeginnerCars {
    telemetry: TheBeginnerCarVehicleTelemetry,
    builds: Vec<TheBeginnerCar>,
    queued_auto_scripts: Vec<AutoScript>
}

impl TheBeginnerCars {
    pub fn new(the_beginner_cars_metadata: Vec<DeviceMetadata>) -> Self {
        let telemetry = TheBeginnerCarVehicleTelemetry::new();

        let builds = the_beginner_cars_metadata
            .into_iter()
            .map(|metadata| TheBeginnerCar::new(metadata.clone(), telemetry.clone()))
            .collect();

        Self { telemetry, builds, queued_auto_scripts: vec![] }
    }

    pub async fn run(self) {
        let mut tasks = tokio::task::JoinSet::new();

        for mut build in self.builds {
            tasks.spawn(async move {
                build.run().await;
            });
        }

        while let Some(result) = tasks.join_next().await {
            if let Err(error) = result {
                eprintln!("A car task failed: {error}");
            }
        }
    }

    pub fn queue_auto_script(&self) {

    }

    pub fn run_auto_script(&self, data: Vec<u8>) {
        let car = self.builds.first().unwrap();
        // car.
    }
 }
