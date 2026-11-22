use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use auto_drive_common_types::LogMessage;
use common_firmware::spi_slave::{SpiSlave, SpiSlaveHardware};
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use log::info;
use the_beginner_car_spi_protocol::{RequestToHardware, ResponseFromHardware};

use crate::{
    auto_script::AutoScript, spawn_thread::spawn_thread,
    the_beginner_car_hardware::TheBeginnerCarHardware, uptime_in_ms::uptime_in_ms,
};

pub struct ServerSpi {
    pub handle: JoinHandle<()>,
}

impl ServerSpi {
    pub fn new<
        SCLK: InputPin + 'static,
        SDI: InputPin + 'static,
        SDO: OutputPin + 'static,
        SRDY: OutputPin + 'static,
        CS: InputPin + 'static,
        MRDY: InputPin + 'static,
    >(
        hardware: SpiSlaveHardware<SCLK, SDI, SDO, SRDY, CS, MRDY>,
        the_beginner_car: Arc<TheBeginnerCarHardware>,
        auto_script: Arc<AutoScript>,
        logs: Arc<Mutex<Vec<LogMessage>>>,
    ) -> anyhow::Result<Self> {
        let handle = spawn_thread(c"server_spi_incoming_messages", 5, 12_000, move || {
            let mut spi = SpiSlave::new(hardware).unwrap();
            spi.listen(|request: RequestToHardware| -> ResponseFromHardware {
                Self::handle_request(request, &the_beginner_car, &auto_script, &logs)
            });
        })?;

        Ok(Self { handle })
    }

    fn handle_request(
        request: RequestToHardware,
        the_beginner_car: &Arc<TheBeginnerCarHardware>,
        auto_script: &Arc<AutoScript>,
        logs: &Arc<Mutex<Vec<LogMessage>>>,
    ) -> ResponseFromHardware {
        match request {
            RequestToHardware::LightOn => println!("LightOn"),
            RequestToHardware::LightOff => println!("LightOff"),
            RequestToHardware::FlushLogs => {
                let logs = {
                    let mut logs = logs.lock().unwrap();
                    std::mem::take(&mut *logs)
                };

                return ResponseFromHardware::Logs { logs };
            }
            RequestToHardware::NotifyPositionFromCamera { position } => {
                the_beginner_car.notify_position_from_camera(position);
            }
            RequestToHardware::NotifyBatteryReadings { voltage } => {
                the_beginner_car.notify_battery_reading(voltage);
            }
            RequestToHardware::NotifyIrSensorReadings { readings } => {
                //
            }
            RequestToHardware::RunAutoScript { data } => {
                info!("run auto script");
                match auto_script.run(data) {
                    Ok(_) => {}
                    Err(err) => log::error!("{}", err),
                }
            }
            RequestToHardware::StopAutoScript => {
                if auto_script.is_running() {
                    dbg!("was running and cancelled");
                    auto_script.cancel()
                }
            } // TODO: RENAME TO CANCEL AutoScript
            RequestToHardware::GetPosition => {
                let position = the_beginner_car.get_position();

                return ResponseFromHardware::Position { position };
            }
            RequestToHardware::Uptime => {
                return ResponseFromHardware::Uptime { ms: uptime_in_ms() };
            }
            RequestToHardware::AutoScriptState => todo!(),
        }

        // pin.set_color(3, 20, 3);
        ResponseFromHardware::Ok
    }
}
