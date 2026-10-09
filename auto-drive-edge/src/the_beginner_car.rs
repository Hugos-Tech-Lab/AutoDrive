use std::{thread, time::Duration};

use metrics::{Gauge, gauge};
use tokio::{net::TcpStream, time::timeout};

use crate::devices_config::{DeviceBuild, DeviceMetadata};

//
pub struct VehicleTelemetry {
    motor_a_set_speed: Gauge,
    motor_b_set_speed: Gauge,
    motor_battery_voltage: Gauge,
    wifi_battery_voltage: Gauge,
}

impl VehicleTelemetry {
    pub fn new() -> Self {
        Self {
            motor_a_set_speed: gauge!("motor_a_set_speed"),
            motor_b_set_speed: gauge!("motor_b_set_speed"),
            motor_battery_voltage: gauge!("motor_battery_voltage"),
            wifi_battery_voltage: gauge!("wifi_battery_voltage"),
        }
    }
}

// Answers questions like:
// Is CPU usage too high?
// What's the average over 5 minutes?
// Hows the Memory?
pub struct SoftwareTelemetry {
    cpu_percentage: Gauge,
}

impl Default for SoftwareTelemetry {
    fn default() -> Self {
        Self {
            cpu_percentage: gauge!("cpu_percentage"),
        }
    }
}

impl SoftwareTelemetry {
    pub fn new() -> Self {
        Self {
            cpu_percentage: gauge!("cpu_percentage"),
        }
    }
}

pub struct TheBeginnerCar {
    build: DeviceBuild,
    mdns_address: String,
    connected: bool,
    software_telemetry: SoftwareTelemetry,
    vehicle_telemetry: VehicleTelemetry,
}

impl TheBeginnerCar {
    pub fn new(device_metadata: DeviceMetadata) -> Self {
        Self {
            build: device_metadata.build,
            mdns_address: device_metadata.mdns_address,
            connected: false,
            software_telemetry: SoftwareTelemetry::new(),
            vehicle_telemetry: VehicleTelemetry::new(),
        }
    }

    pub async fn run(&self) {
        loop {
            let mut stream = match timeout(
                Duration::from_secs(5),
                TcpStream::connect(self.mdns_address.clone()),
            )
            .await
            {
                Ok(Ok(stream)) => {
                    println!("Connected!");
                    stream
                }

                Ok(Err(error)) => {
                    eprintln!("Failed to connect: {error}");
                    thread::sleep(Duration::from_secs(10));
                    continue;
                }

                Err(_) => {
                    eprintln!("Connection timed out after 5 seconds");
                    thread::sleep(Duration::from_secs(10));
                    continue;
                }
            };
        }
    }
}
