use std::{
    sync::{Arc, Mutex, atomic::Ordering},
    time::Duration,
};

use auto_drive_common_types::LogSystemOrigin;
use common_firmware::{logger::init_logging, spi_slave::SpiSlaveHardware};
#[cfg(all(esp_idf_app_compile_time_date, not(esp_idf_app_reproducible_build)))]
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::{
    adc::{
        attenuation::DB_12,
        oneshot::{AdcChannelDriver, AdcDriver, config::AdcChannelConfig},
    },
    delay::FreeRtos,
};
use the_beginner_car_spi_protocol::{Event, Position};

use crate::{
    auto_script::AutoScript,
    server_spi::ServerSpi,
    the_beginner_car_hardware::{
        KalmanFilter, KalmanFilterIterateState, TheBeginnerCarHardware,
        ir_sensor_array::{IrSensorArray, IrSensorArrayData},
        led::Led,
        wheel::{Wheel, motor::Motor, tachometer::Tachometer},
    },
};

use esp_idf_sys::esp_reset_reason;
use log::info;
pub mod auto_script;
pub mod calibration;
pub mod server_spi;
pub mod the_beginner_car_hardware;

pub fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    unsafe {
        let config = esp_idf_sys::esp_vfs_eventfd_config_t { max_fds: 5 };
        esp_idf_sys::esp_vfs_eventfd_register(&config);
    }

    let reason = unsafe { esp_reset_reason() };
    println!("Last reset reason: {:?}", reason);

    let log_receiver = init_logging(LogSystemOrigin::WasmMcu);

    info!("starting");
    let peripherals = Peripherals::take()?;

    let wheel_right_tachometer = Tachometer::new(peripherals.pins.gpio5.into());
    let wheel_left_tachometer = Tachometer::new(peripherals.pins.gpio6.into());

    let mut kalman_filter = KalmanFilter::new(
        wheel_left_tachometer.clone(),
        wheel_right_tachometer.clone(),
    );

    let motor_left = Motor::new(
        peripherals.pins.gpio7.into(),
        peripherals.pins.gpio18.into(),
        peripherals.ledc.channel0,
        peripherals.ledc.timer1,
        wheel_left_tachometer.clone(),
    );

    let motor_right = Motor::new(
        peripherals.pins.gpio11.into(),
        peripherals.pins.gpio10.into(),
        peripherals.ledc.channel1,
        peripherals.ledc.timer2,
        wheel_right_tachometer.clone(),
    );

    // let wheel_right_tachometer = Tachometer::new(peripherals.pins.gpio13.into());

    let wheel_left = Wheel::new(motor_left, wheel_left_tachometer);
    let wheel_right = Wheel::new(motor_right, wheel_right_tachometer);

    if reason == 9 {
        // pin.set_color(9, 0, 200);
        FreeRtos::delay_ms(1000);
    }

    let ir_sensor_array = Arc::new(IrSensorArray {}); // TODO construct with new 

    let the_beginner_car = Arc::new(Mutex::new(TheBeginnerCarHardware::new(
        wheel_left,
        wheel_right,
        // led_right,
        ir_sensor_array.clone(),
        kalman_filter.clone(),
    )));

    // let latest_position: Arc<Mutex<Option<Position>>> = Arc::new(Mutex::new(None));
    let events: Arc<Mutex<Vec<Event>>> = Arc::new(Mutex::new(vec![]));

    let auto_script = Arc::new(AutoScript::new(the_beginner_car.clone(), {
        let events = events.clone();
        move || {
            let mut events = events.lock().unwrap();
            events.push(Event::AutoScriptStopped);
        }
    })?);

    std::thread::Builder::new()
        .name("position".to_string())
        .spawn({
            let events = events.clone();

            move || {
                let mut kalman_filter_state = kalman_filter.create_iterate_state();

                loop {
                    let _ = kalman_filter.iterate(&mut kalman_filter_state);
                    // TODO: SAFETY?? check position isn't out of bounds or anything like that.
                    // {

                    // let latest_position =
                    // }
                    // let Some(position) {
                    //     //
                    // }

                    // if let Some(position) = position {
                    //     {
                    //         let mut event_guard = events.lock().unwrap();
                    //         let timestamp_unix_epoch_ms = std::time::SystemTime::now()
                    //             .duration_since(std::time::UNIX_EPOCH)
                    //             .expect("Time went backwards")
                    //             .as_millis()
                    //             as u64;

                    //         // event_guard.push(Position {
                    //         //     x: position.x,
                    //         //     y: position.y,
                    //         //     rotation_radians: position.rotation_radians,
                    //         //     timestamp_unix_epoch_ms,
                    //         // });
                    //     }
                    //     // TODO: check if out of bounds
                    // }

                    FreeRtos::delay_ms(100);
                }
            }
        })
        .unwrap();

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

    //

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
    // reading.join().unwrap();
    Ok(())
}
