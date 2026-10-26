use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use common_firmware::spi_slave::{SpiSlave, SpiSlaveHardware};
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use log::info;
use the_beginner_car_spi_protocol::{Event, RequestToHardware, ResponseFromHardware};

use crate::{
    auto_script::AutoScript,
    hardware::{RequestOrigin, TheBeginnerCarHardware},
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
        the_beginner_car: Arc<Mutex<TheBeginnerCarHardware>>,
        auto_script: Arc<AutoScript>,
        events: Arc<Mutex<Vec<Event>>>,
    ) -> anyhow::Result<Self> {
        let handle = std::thread::Builder::new()
            .stack_size(12_000)
            .name("read_incoming_messages".to_string())
            .spawn(move || {
                let mut spi = SpiSlave::new(hardware).unwrap();
                spi.listen(|request: RequestToHardware| -> ResponseFromHardware {
                    Self::handle_request(request, &the_beginner_car, &auto_script, &events)
                });
            })?;

        Ok(Self { handle })
    }

    fn handle_request(
        request: RequestToHardware,
        the_beginner_car: &Arc<Mutex<TheBeginnerCarHardware>>,
        auto_script: &Arc<AutoScript>,
        events: &Arc<Mutex<Vec<Event>>>,
    ) -> ResponseFromHardware {
        match request {
            RequestToHardware::LightOn => println!("LightOn"),
            RequestToHardware::LightOff => println!("LightOff"),
            RequestToHardware::SetMotorASpeed(speed) => {
                let mut the_beginner_car = the_beginner_car.lock().unwrap();
                the_beginner_car.set_wheel_a_speed(speed, RequestOrigin::Manual);
            }
            RequestToHardware::SetMotorBSpeed(speed) => {
                let mut the_beginner_car = the_beginner_car.lock().unwrap();
                the_beginner_car.set_wheel_b_speed(speed, RequestOrigin::Manual);
            }
            RequestToHardware::DrainEvents => {
                let events = {
                    let mut events = events.lock().unwrap();
                    std::mem::take(&mut *events)
                };

                return ResponseFromHardware::Events { events };
            }
            RequestToHardware::NotifyPositionFromCamera { x, y } => {
                println!("{x}, {y}.. pos from camera")
            }
            RequestToHardware::NotifyBatteryReadings { voltage } => {
                let mut the_beginner_car = the_beginner_car.lock().unwrap();
                the_beginner_car.notify_battery_reading(voltage);
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
                } else {
                    let mut events = events.lock().unwrap();
                    events.push(Event::AutoScriptStopped);
                }
            } // TODO: RENAME TO CANCEL AutoScript
        }

        // pin.set_color(3, 20, 3);
        ResponseFromHardware::Ok
    }
}
