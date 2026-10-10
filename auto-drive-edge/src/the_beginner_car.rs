use std::{thread, time::Duration};

use opentelemetry::{
    KeyValue,
    metrics::{Gauge, Meter},
};
use the_beginner_car_tcp_protocol::{
    TheBeginnerCarIncomingMessages, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, lookup_host},
    time::{sleep, timeout},
};

use crate::{
    devices_config::{DeviceBuild, DeviceMetadata},
    the_beginner_cars::TheBeginnerCarVehicleTelemetry,
};

//
// pub struct VehicleTelemetry {
//     // motor_a_set_speed: Gauge<f64>,
//     // motor_b_set_speed: Gauge<f64>,
//     motor_battery_voltage: Gauge<f64>,
//     // wifi_battery_voltage: Gauge<f64>,
// }

// impl VehicleTelemetry {
//     pub fn new(meter: Meter, identifier: [KeyValue; 1]) -> Self {

//         motor_battery_voltage.record(2.0, &identifier);

//         Self {
//             motor_battery_voltage,
//         }
//     }
// }

// Answers questions like:
// Is CPU usage too high?
// What's the average over 5 minutes?
// Hows the Memory?
// pub struct SoftwareTelemetry {
//     cpu_percentage: Gauge,
// }

// impl Default for SoftwareTelemetry {
//     fn default() -> Self {
//         Self {
//             cpu_percentage: gauge!("cpu_percentage"),
//         }
//     }
// }

// impl SoftwareTelemetry {
//     pub fn new() -> Self {
//         Self {
//             cpu_percentage: gauge!("cpu_percentage"),
//         }
//     }
// }

pub struct TheBeginnerCar {
    build: DeviceBuild,
    mdns_address: String,
    connected: bool,
    vehicle_telemetry: TheBeginnerCarVehicleTelemetry, // software_telemetry: SoftwareTelemetry,
                                                       // vehicle_telemetry: VehicleTelemetry,
                                                       // telemetry_attrobute: []
}

impl TheBeginnerCar {
    pub fn new(
        device_metadata: DeviceMetadata,
        vehicle_telemetry: TheBeginnerCarVehicleTelemetry,
    ) -> Self {
        Self {
            build: device_metadata.build,
            mdns_address: device_metadata.mdns_address,
            connected: false,
            vehicle_telemetry, // software_telemetry: SoftwareTelemetry::new(),
                               // vehicle_telemetry: VehicleTelemetry::new(),
        }
    }

    pub async fn run(&self) {
        loop {
            let m_dns = self.mdns_address.as_str();

            let host = format!("{m_dns}:8080");
            let Ok(mut addresses) = lookup_host(&host).await else {
                println!("Could not find {host}. Trying again in 10 seconds");
                thread::sleep(Duration::from_secs(10));
                continue;
            };

            let addr = addresses
                .next()
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "No socket addresses found")
                })
                .unwrap();

            let mut stream = match timeout(Duration::from_secs(5), TcpStream::connect(addr.clone()))
                .await
            {
                Ok(Ok(stream)) => {
                    println!("Connected to {host}.");
                    stream
                }

                Ok(Err(error)) => {
                    let addr = addr.clone();
                    eprintln!(
                        "Failed to connect to {addr} error: {error}. Trying again in 10 seconds"
                    );
                    thread::sleep(Duration::from_secs(10));
                    continue;
                }

                Err(_) => {
                    eprintln!("Connection timed out after 5 seconds. Trying again in 10 seconds");
                    thread::sleep(Duration::from_secs(10));
                    continue;
                }
            };

            loop {
                let message = TheBeginnerCarIncomingMessages::GetBatteryReading;

                let bytes = postcard::to_stdvec(&message).unwrap();
                let len = u32::try_from(bytes.len()).unwrap();

                // Send [4-byte length][postcard message]
                stream.write_all(&len.to_be_bytes()).await.unwrap();
                stream.write_all(&bytes).await.unwrap();

                let mut len_bytes = [0u8; 4];
                stream.read_exact(&mut len_bytes).await.unwrap();
                let len = u32::from_be_bytes(len_bytes) as usize;

                let mut message_buf = vec![0u8; len as usize];
                stream.read_exact(&mut message_buf).await.unwrap();

                // Deserialize
                let message: TheBeginnerCarOutgoingMessages =
                    postcard::from_bytes(&message_buf).unwrap();
                self.handle_response(message);


                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    pub fn handle_response(&self, message: TheBeginnerCarOutgoingMessages) {
        match message {
            TheBeginnerCarOutgoingMessages::Logs { logs } => todo!(),
            TheBeginnerCarOutgoingMessages::SystemState {} => todo!(),
            TheBeginnerCarOutgoingMessages::MotorASpeedUpdated { speed } => todo!(),
            TheBeginnerCarOutgoingMessages::MotorBSpeedUpdated { speed } => todo!(),
            TheBeginnerCarOutgoingMessages::WasmInstalled => todo!(),
            TheBeginnerCarOutgoingMessages::WasmRunning => todo!(),
            TheBeginnerCarOutgoingMessages::BatteryReading { voltage } => {
                self.vehicle_telemetry
                    .motor_battery_voltage
                    .record(voltage.into(), &[KeyValue::new("model", "TheBeginnerCar"), KeyValue::new("build", "BuildA")]);
            }
        }
    }
}
