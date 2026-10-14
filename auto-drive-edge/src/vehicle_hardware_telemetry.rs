use std::sync::Arc;

use opentelemetry::{KeyValue, global, metrics::Gauge};
use the_beginner_car_tcp_protocol::TheBeginnerCarOutgoingMessages;
use tokio::{sync::broadcast, task::JoinSet};

use crate::{devices_config::DeviceBuild, physical_hardware::PhysicalHardware};

#[derive(Debug, Clone)]
struct VehicleHardwareTelemetryGauges {
    motor_battery_voltage: Gauge<f64>,
    mcu_battery_voltage: Gauge<f64>,
}

impl VehicleHardwareTelemetryGauges {
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
            mcu_battery_voltage,
        }
    }
}

pub struct VehicleHardwareTelemetry {
    the_beginner_car_watcher_handles: JoinSet<()>,
}

impl VehicleHardwareTelemetry {
    pub fn new(hardware: Arc<PhysicalHardware>) -> Self {
        let gauges = VehicleHardwareTelemetryGauges::new();

        let mut the_beginner_car_watcher_handles = tokio::task::JoinSet::new();
        for the_beginner_car in hardware.the_beginner_cars() {
            the_beginner_car_watcher_handles.spawn({
                let device_build = the_beginner_car.build.clone();
                let incoming_rx = the_beginner_car.incoming_tx.subscribe();
                let gauges = gauges.clone();
                async move {
                    Self::the_beginner_car_long_lived_watcher(device_build, incoming_rx, gauges)
                        .await;
                }
            });
        }

        Self {
            the_beginner_car_watcher_handles,
        }
    }

    async fn the_beginner_car_long_lived_watcher(
        build: DeviceBuild, // 
        mut incoming_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
        gauges: VehicleHardwareTelemetryGauges,
    ) {
        let attributes = &[
            KeyValue::new("model", "TheBeginnerCar"),
            KeyValue::new("build", "BuildA"), // TODO this should be from build
        ];

        loop {
            match incoming_rx.recv().await {
                Ok(message) => match message {
                    TheBeginnerCarOutgoingMessages::BatteryReading { motor_battery_voltage, mcu_battery_voltage } => {
                        gauges
                            .motor_battery_voltage
                            .record(motor_battery_voltage.into(), attributes);
                        gauges
                            .mcu_battery_voltage
                            .record(mcu_battery_voltage.into(), attributes);
                    },
                    // TheBeginnerCarOutgoingMessages::IRSensorAReading { value } => {
                    //     dbg!(value);
                    // }
                    _ => {}
                },
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    eprintln!("Car watcher lagged; skipped {skipped} messages");
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    }
}
