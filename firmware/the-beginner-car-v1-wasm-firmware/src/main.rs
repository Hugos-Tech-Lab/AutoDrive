use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU16, Ordering},
    },
    time::Duration,
};

use common_firmware::spi_slave::SpiSlaveHardware;
#[cfg(all(esp_idf_app_compile_time_date, not(esp_idf_app_reproducible_build)))]
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::{
    adc::{
        attenuation::DB_12,
        oneshot::{AdcChannelDriver, AdcDriver, config::AdcChannelConfig},
    },
    delay::FreeRtos,
};
use the_beginner_car_spi_protocol::Event;

use crate::{
    auto_script::AutoScript, hardware::{TheBeginnerCarHardware, led::Led, motor::Motor}, logger::init_logging, server_spi::ServerSpi,
};

use esp_idf_sys::esp_reset_reason;
use log::info;
pub mod auto_script;
pub mod hardware;
pub mod logger;
pub mod server_spi;

pub fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    unsafe {
        let config = esp_idf_sys::esp_vfs_eventfd_config_t { max_fds: 5 };
        esp_idf_sys::esp_vfs_eventfd_register(&config);
    }

    let reason = unsafe { esp_reset_reason() };
    println!("Last reset reason: {:?}", reason);

    let log_receiver = init_logging();

    info!("starting");
    let peripherals = Peripherals::take()?;

    let motor_a = Motor::new(
        peripherals.pins.gpio7.into(),
        peripherals.pins.gpio8.into(),
        peripherals.ledc.channel0,
        peripherals.ledc.timer1,
    );
    let motor_b = Motor::new(
        peripherals.pins.gpio11.into(),
        peripherals.pins.gpio10.into(),
        peripherals.ledc.channel1,
        peripherals.ledc.timer2,
    );
    let led_left = Led::new(peripherals.pins.gpio18.into());
    let led_right = Led::new(peripherals.pins.gpio9.into());

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
        led_left,
        led_right,
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
                    let utmost_front_utmost_left_latest_reading =
                        ((pin_3.read_raw().unwrap() as u32 * u16::MAX as u32) / 4095) as u16;
                    let utmost_front_utmost_right_latest_reading =
                        ((pin_2.read_raw().unwrap() as u32 * u16::MAX as u32) / 4095) as u16;

                    // dbg!(led_b_latest_reading);
                    ir_utmost_front_utmost_left_latest_reading
                        .store(utmost_front_utmost_left_latest_reading, Ordering::Relaxed);
                    ir_utmost_front_utmost_right_latest_reading
                        .store(utmost_front_utmost_right_latest_reading, Ordering::Relaxed);

                    if last_flush.elapsed() >= Duration::from_millis(100) {
                        let mut events = events.lock().unwrap();

                        events.push(Event::InfraredLedReadings {
                            utmost_front_utmost_left: utmost_front_utmost_left_latest_reading,
                            utmost_front_utmost_right: utmost_front_utmost_right_latest_reading,
                        });

                        last_flush = std::time::Instant::now();
                    }

                    // dbg!(led_a_latest_reading);
                    FreeRtos::delay_ms(1);
                }
            }
        })
        .unwrap();

    let spi_hardware = SpiSlaveHardware {
        spi_2: peripherals.spi2,
        a_sclk: peripherals.pins.gpio15,
        b_sdi: peripherals.pins.gpio23,
        c_sdo: peripherals.pins.gpio22,
        d_slave_ready_pin: peripherals.pins.gpio21,
        e_cs: peripherals.pins.gpio20,
        f_master_ready_pin: peripherals.pins.gpio19,
    };

    let server_spi = ServerSpi::new(spi_hardware, the_beginner_car, auto_script, events)?;

    server_spi.handle.join().unwrap();
    reading.join().unwrap();
    Ok(())
}
