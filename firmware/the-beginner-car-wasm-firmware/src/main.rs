use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU16, Ordering},
    },
    thread,
    time::Duration,
};

use common_firmware::{
    get_mac_address::{format_mac_address, get_mac_address},
    spi_slave::SpiSlave,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        adc::{
            attenuation::DB_12,
            oneshot::{AdcChannelDriver, AdcDriver, config::AdcChannelConfig},
        },
        delay::{FreeRtos, TICK_RATE_HZ},
    },
};
#[cfg(all(esp_idf_app_compile_time_date, not(esp_idf_app_reproducible_build)))]
use esp_idf_svc::{hal::peripherals::Peripherals, nvs::EspDefaultNvsPartition};
use the_beginner_car_spi_protocol::{Event, RequestToHardware, ResponseFromHardware};

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

    let log_receiver = init_logging();

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
        FreeRtos::delay_ms(1000);
    }

    let ir_utmost_front_utmost_left_latest_reading = Arc::new(AtomicU16::new(0));
    let ir_utmost_front_utmost_right_latest_reading = Arc::new(AtomicU16::new(0));

    let the_beginner_car = Arc::new(Mutex::new(TheBeginnerCarHardware::new(
        motor_a,
        motor_b,
        ir_utmost_front_utmost_left_latest_reading.clone(),
        ir_utmost_front_utmost_right_latest_reading.clone(),
    )));

    let events: Arc<Mutex<Vec<Event>>> = Arc::new(Mutex::new(vec![]));

    let auto_script = Arc::new(AutoScript::new(the_beginner_car.clone(), {
        let events = events.clone();
        move || {
            let mut events = events.lock().unwrap();
            events.push(Event::AutoScriptStopped);
        }
    })?);

    std::thread::Builder::new()
        .stack_size(2_000)
        .name("flush_logs".to_string())
        .spawn({
            let events = events.clone();
            move || {
                let mut pending_logs = Vec::new();
                let mut last_flush = std::time::Instant::now();

                loop {
                    match log_receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                        Ok(log) => pending_logs.push(log),
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                            dbg!();
                            panic!();
                        }
                    }

                    if last_flush.elapsed() >= std::time::Duration::from_secs(1) {
                        let mut events = events.lock().unwrap();
                        events.extend(pending_logs.drain(..).map(|message| Event::Log { message }));

                        last_flush = std::time::Instant::now();
                    }
                }
            }
        })
        .unwrap();

    let reading = std::thread::Builder::new()
        .stack_size(2_000)
        .name("sensor_readings".to_string())
        .spawn({
            let events = events.clone();
            move || {
                let adc = AdcDriver::new(peripherals.adc1).unwrap();
                let config = AdcChannelConfig {
                    attenuation: DB_12,
                    ..Default::default()
                };
                let mut pin_3 =
                    AdcChannelDriver::new(&adc, peripherals.pins.gpio3, &config).unwrap();
                let mut pin_2 =
                    AdcChannelDriver::new(&adc, peripherals.pins.gpio2, &config).unwrap();

                let mut last_flush = std::time::Instant::now();

                loop {
                    // TODO: this polling could be based on the speed of the wheels
                    let utmost_front_utmost_left_latest_reading = ((pin_3.read_raw().unwrap() as u32 * u16::MAX as u32) / 4095) as u16;
                    let utmost_front_utmost_right_latest_reading = ((pin_2.read_raw().unwrap() as u32 * u16::MAX as u32) / 4095) as u16;

                    // dbg!(led_b_latest_reading);
                    ir_utmost_front_utmost_left_latest_reading.store(utmost_front_utmost_left_latest_reading, Ordering::Relaxed);
                    ir_utmost_front_utmost_right_latest_reading.store(utmost_front_utmost_right_latest_reading, Ordering::Relaxed);
                    if last_flush.elapsed() >= Duration::from_millis(100) {
                        let mut events = events.lock().unwrap();

                        events.push(Event::InfraredLedReadings {
                            utmost_front_utmost_left: utmost_front_utmost_left_latest_reading,
                            utmost_front_utmost_right: utmost_front_utmost_right_latest_reading,
                        });

                        last_flush = std::time::Instant::now();
                    }

                    dbg!(utmost_front_utmost_left_latest_reading);
                    // dbg!(led_a_latest_reading);
                    FreeRtos::delay_ms(200);
                }
            }
        })
        .unwrap();

    let thread_receive_incoming_messages = std::thread::Builder::new()
        .stack_size(12_000)
        .name("read_incoming_messages".to_string())
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
                    RequestToHardware::DrainEvents => {
                        let events = {
                            let mut events = events.lock().unwrap();
                            std::mem::take(&mut *events)
                        };

                        return ResponseFromHardware::Events { events };
                    }
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
            });
        })?;

    thread_receive_incoming_messages.join().unwrap();
    reading.join().unwrap();
    Ok(())
}
