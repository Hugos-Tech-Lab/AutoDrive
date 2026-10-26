use std::{sync::mpsc::SyncSender, thread, time::Duration};
pub mod battery_monitor;

use auto_drive_common_types::LogSystemOrigin;
use common_firmware::{logger::init_logging, spi_master::SpiMaster};
use esp_idf_hal::adc::oneshot::AdcDriver;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::gpio::{PinDriver, Pull},
    mdns::EspMdns,
    wifi::{BlockingWifi, EspWifi},
};
#[cfg(all(esp_idf_app_compile_time_date, not(esp_idf_app_reproducible_build)))]
use esp_idf_svc::{hal::peripherals::Peripherals, nvs::EspDefaultNvsPartition};
use futures::{FutureExt, executor::LocalPool, task::LocalSpawnExt};
use the_beginner_car_spi_protocol::{RequestToHardware, ResponseFromHardware};
use the_beginner_car_tcp_protocol::TheBeginnerCarOutgoingMessages;

use crate::{
    battery_monitor::battery_monitor::Battery, connect_to_wifi::connect_to_wifi, tcp_server::accept,
};

use esp_idf_sys::esp_reset_reason;
use log::info;
pub mod connect_to_wifi;
pub mod inter_thread;
pub mod tcp_server;
pub mod utils;

pub struct HardwareMessage {
    request: RequestToHardware,
    response_tx: SyncSender<anyhow::Result<ResponseFromHardware>>,
}

pub fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    unsafe {
        let config = esp_idf_sys::esp_vfs_eventfd_config_t { max_fds: 16 };

        let ret = esp_idf_sys::esp_vfs_eventfd_register(&config);

        assert_eq!(
            ret,
            esp_idf_sys::ESP_OK,
            "esp_vfs_eventfd_register failed: {:?}",
            ret
        );
    }

    let reason = unsafe { esp_reset_reason() };
    println!("Last reset reason: {:?}", reason);

    let log_receiver = init_logging(LogSystemOrigin::WifiMcu);
    info!("starting");
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    if reason == 9 {
        thread::sleep(Duration::from_secs(1));
    }

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    connect_to_wifi(&mut wifi)?;

    let mut mdns = EspMdns::take()?;
    mdns.set_hostname("the_beginner_car_build_a")?;

    info!("setting host name");
    let (hardware_sender, hardware_receiver) =
        std::sync::mpsc::sync_channel::<HardwareMessage>(100);

    let (sender_tx, receiver_tx) = flume::bounded(1000);

    let tcp_handle = std::thread::Builder::new()
        .name("tcp_handle".to_string())
        .stack_size(40 * 1000)
        .spawn({
            let hardware_sender = hardware_sender.clone();
            || {
                let mut local_executor = LocalPool::new();
                let spawner = local_executor.spawner();
                local_executor
                    .spawner()
                    .spawn_local(
                        async move {
                            accept(spawner, hardware_sender, receiver_tx).await.unwrap();

                            Result::<_, anyhow::Error>::Ok(())
                        }
                        .map(Result::unwrap),
                    )
                    .unwrap();

                local_executor.run();
            }
        })
        .unwrap();

    std::thread::Builder::new()
        .stack_size(2_000)
        .name("flush_logs".to_string())
        .spawn({
            let sender_tx = sender_tx.clone();
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
                        for log in pending_logs.drain(..) {
                            sender_tx
                                .send(TheBeginnerCarOutgoingMessages::Log { log })
                                .unwrap();
                        }
                        last_flush = std::time::Instant::now();
                    }
                }
            }
        })
        .unwrap();

    let drain_events_handle = std::thread::Builder::new()
        .name("drain_events_handle".to_string())
        .stack_size(4 * 1000)
        .spawn({
            let hardware_sender = hardware_sender.clone();
            let sender_tx = sender_tx.clone();
            move || {
                loop {
                    let (sender, receiver) = std::sync::mpsc::sync_channel::<
                        Result<ResponseFromHardware, anyhow::Error>,
                    >(1);

                    hardware_sender
                        .send(HardwareMessage {
                            request: RequestToHardware::DrainEvents,
                            response_tx: sender,
                        })
                        .unwrap();

                    let receive = match receiver.recv_timeout(Duration::from_secs(2)) {
                        Ok(Ok(response)) => response,
                        Ok(Err(e)) => {
                            log::error!("{e}");
                            thread::sleep(Duration::from_secs(1));
                            continue;
                        }
                        Err(e) => {
                            log::error!("{e}");
                            continue;
                        }
                    };

                    match receive {
                        ResponseFromHardware::Events { events } => {
                            for event in events {
                                match event {
                                    the_beginner_car_spi_protocol::Event::Log { message } => {
                                        sender_tx
                                            .send(TheBeginnerCarOutgoingMessages::Log {
                                                log: message,
                                            })
                                            .unwrap()
                                    } // TODO: this should send vec
                                    the_beginner_car_spi_protocol::Event::InfraredLedReadings { back_center, front_left, front_center, front_right  } => {
                                        // dbg!(value);
                                        sender_tx
                                            .send(TheBeginnerCarOutgoingMessages::InfraredLedReadings { back_center, front_left, front_center, front_right })
                                            .unwrap()
                                    }
                                    the_beginner_car_spi_protocol::Event::AutoScriptStopped => {
                                        sender_tx
                                            .send(TheBeginnerCarOutgoingMessages::AutoScriptStopped)
                                            .unwrap()
                                    }
                                }
                            }
                        }
                        _ => {
                            print!("unexpected response")
                        }
                    }

                    thread::sleep(Duration::from_secs(1));
                }
            }
        })
        .unwrap();

    let battery_poll_handle = std::thread::Builder::new()
        .name("battery_poll_handle".to_string())
        .stack_size(4 * 1000)
        .spawn({
            let hardware_sender = hardware_sender.clone();
            let sender_tx = sender_tx.clone();
            move || {
                let adc = AdcDriver::new(peripherals.adc1).unwrap();
                let mut battery = Battery::new(&adc, peripherals.pins.gpio4).unwrap();

                loop {
                    let motor_battery_reading = battery.read().unwrap();

                    {
                        sender_tx
                            .send(TheBeginnerCarOutgoingMessages::BatteryReading {
                                voltage: motor_battery_reading.voltage,
                            })
                            .unwrap();
                    }

                    let (sender, receiver) = std::sync::mpsc::sync_channel::<
                        Result<ResponseFromHardware, anyhow::Error>,
                    >(1);

                    hardware_sender
                        .send(HardwareMessage {
                            request: RequestToHardware::NotifyBatteryReadings {
                                voltage: motor_battery_reading.voltage,
                            },
                            response_tx: sender,
                        })
                        .unwrap();

                    let receive = match receiver.recv_timeout(Duration::from_secs(2)) {
                        Ok(Ok(response)) => response,
                        Ok(Err(e)) => {
                            log::error!("{e}");
                            thread::sleep(Duration::from_secs(1));
                            continue;
                        }
                        Err(e) => {
                            log::error!("{e}");
                            continue;
                        }
                    };

                    if !matches!(receive, ResponseFromHardware::Ok) {
                        log::error!("response other than ok?"); // Received response other than ok
                    }

                    // TODO: keep track of battery reading for the PC

                    thread::sleep(Duration::from_secs(1));
                }
            }
        })?;

    let spi_handle = std::thread::Builder::new()
        .name("spi_driver".into())
        .stack_size(16 * 1024)
        .spawn(move || {
            let mut spi = SpiMaster::new(
                // TODO: cleanup this similar to SpiSlave.
                peripherals.spi2,
                peripherals.pins.gpio15.into(),
                peripherals.pins.gpio23.into(),
                peripherals.pins.gpio22.into(),
                PinDriver::input(peripherals.pins.gpio21, Pull::Down)
                    .unwrap()
                    .into(),
                peripherals.pins.gpio20.into(),
                PinDriver::output(peripherals.pins.gpio19).unwrap().into(),
                5_000_000,
            )
            .unwrap();

            loop {
                match hardware_receiver.recv() {
                    Ok(message) => {
                        let res: Result<ResponseFromHardware, anyhow::Error> =
                            spi.send_request(&message.request);

                        let _ = message.response_tx.send(res);
                    }
                    Err(err) => {
                        info!("Hardware receiver closed: {:?}", err);
                        break;
                    }
                }
            }
        })?;

    info!("init done");

    spi_handle.join().unwrap();
    tcp_handle.join().unwrap();
    battery_poll_handle.join().unwrap();
    drain_events_handle.join().unwrap();
    Ok(())
}
