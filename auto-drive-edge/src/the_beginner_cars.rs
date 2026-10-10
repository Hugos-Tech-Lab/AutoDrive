use opentelemetry::{global, metrics::Gauge};

use crate::{devices_config::DeviceMetadata, the_beginner_car::TheBeginnerCar};

#[derive(Clone)]
pub struct TheBeginnerCarVehicleTelemetry {
    // motor_a_set_speed: Gauge<f64>,
    // motor_b_set_speed: Gauge<f64>,
    pub motor_battery_voltage: Gauge<f64>,
    pub mcu_battery_voltage: Gauge<f64>,
}

impl TheBeginnerCarVehicleTelemetry {
    pub fn new() -> Self {
        let meter = global::meter("auto-drive-edge");
        let motor_battery_voltage: Gauge<f64> = meter
            .f64_gauge("motor_battery_voltage")
            .with_unit("V")
            .with_description("The motor battery voltage")
            .build();
        let mcu_battery_voltage: Gauge<f64> = meter
            .f64_gauge("mcu_battery_voltage")
            .with_unit("V")
            .with_description("The MCU battery voltage")
            .build();


        Self {
            motor_battery_voltage,
            mcu_battery_voltage
        }
    }
}

pub struct TheBeginnerCars {
    telemetry: TheBeginnerCarVehicleTelemetry,
    builds: Vec<TheBeginnerCar>,
}

impl TheBeginnerCars {
    pub fn new(the_beginner_cars_metadata: Vec<DeviceMetadata>) -> Self {
        let telemetry = TheBeginnerCarVehicleTelemetry::new();

        let builds = the_beginner_cars_metadata
            .into_iter()
            .map(|metadata| TheBeginnerCar::new(metadata.clone(), telemetry.clone()))
            .collect();

        Self { telemetry, builds }
    }

    pub async fn run(self) {
        let mut tasks = tokio::task::JoinSet::new();

        for build in self.builds {
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
}
