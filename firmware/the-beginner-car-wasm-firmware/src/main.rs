use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use common_firmware::{
    get_mac_address::{format_mac_address, get_mac_address},
    spi_slave::SpiSlave,
};
use esp_idf_svc::eventloop::EspSystemEventLoop;
#[cfg(all(esp_idf_app_compile_time_date, not(esp_idf_app_reproducible_build)))]
use esp_idf_svc::{hal::peripherals::Peripherals, nvs::EspDefaultNvsPartition};
use the_beginner_car_spi_protocol::{RequestToHardware, ResponseFromHardware};

use crate::{
    auto_script::AutoScript,
    hardware::{RequestOrigin, TheBeginnerCarHardware, motor::Motor},
    logger::init_logging,
};

use esp_idf_sys::esp_reset_reason;
use log::info;
pub mod auto_script;
pub mod esp_app_desc_2;
pub mod hardware;
pub mod inter_thread;
pub mod logger;

pub fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    unsafe {
        let config = esp_idf_sys::esp_vfs_eventfd_config_t { max_fds: 5 };
        esp_idf_sys::esp_vfs_eventfd_register(&config);
    }

    let reason = unsafe { esp_reset_reason() };
    println!("Last reset reason: {:?}", reason);

    let addr = format_mac_address(&get_mac_address()?);

    let (log_sender, log_receiver) = init_logging();

    info!("starting");
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let motor_a = Motor::new(
        peripherals.pins.gpio15.into(),
        peripherals.pins.gpio23.into(),
        peripherals.ledc.channel0,
        peripherals.ledc.timer1,
    );
    let motor_b = Motor::new(
        peripherals.pins.gpio21.into(),
        peripherals.pins.gpio22.into(),
        peripherals.ledc.channel1,
        peripherals.ledc.timer2,
    );

    // pin.set_color(20, 2, 2);
    if reason == 9 {
        // pin.set_color(9, 0, 200);
        thread::sleep(Duration::from_secs(1));
    }

    // let mut pin = AdcChannelDriver::new(&adc, peripherals.pins.gpio3, &config)?;

    let the_beginner_car = Arc::new(Mutex::new(TheBeginnerCarHardware::new(motor_a, motor_b)));

    let auto_script = Arc::new(AutoScript::new(the_beginner_car.clone())?);

    let thread_receive_incoming_messages =
        std::thread::Builder::new()
            .stack_size(12_000)
            .spawn(move || {
                let mut spi = SpiSlave::new(
                    peripherals.spi2,
                    peripherals.pins.gpio4,
                    peripherals.pins.gpio5,
                    peripherals.pins.gpio6,
                    peripherals.pins.gpio7,
                    peripherals.pins.gpio0,
                    peripherals.pins.gpio1,
                )
                .unwrap();

                // pin.set_color(3, 20, 3);
                spi.listen(|request: RequestToHardware| -> ResponseFromHardware {
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
                        RequestToHardware::Logs => println!("Logs"),
                        RequestToHardware::NotifyPositionFromCamera { x, y } => {
                            println!("pos from camera")
                        }
                        RequestToHardware::NotifyBatteryReadings {
                            motor_battery,
                            mcu_battery,
                        } => {
                            let mut the_beginner_car = the_beginner_car.lock().unwrap();
                            the_beginner_car.notify_battery_readings(motor_battery, mcu_battery);
                        }
                        RequestToHardware::RunAutoScript { data } => {
                            auto_script.run(data).unwrap();
                        }
                    }

                    // pin.set_color(3, 20, 3);
                    ResponseFromHardware::Ok
                });
            })?;

    thread_receive_incoming_messages.join().unwrap();

    Ok(())
}
