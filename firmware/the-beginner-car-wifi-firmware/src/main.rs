use std::{sync::mpsc::SyncSender, thread, time::Duration};
pub mod battery_monitor;

use common_firmware::spi_master::SpiMaster;
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
use the_beginner_car_tcp_protocol::{LogLevel, LogMessage, LogSystemOrigin, TheBeginnerCarOutgoingMessages};

use crate::{
    battery_monitor::battery_monitor::Battery, connect_to_wifi::connect_to_wifi,
    logger::init_logging, tcp_server::accept,
};

use esp_idf_sys::esp_reset_reason;
use log::info;
pub mod connect_to_wifi;
pub mod esp_app_desc_2;
pub mod inter_thread;
pub mod logger;
pub mod tcp_server;
pub mod utils;

pub fn print_memory_stats() {
    unsafe {
        let internal_free =
            esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_INTERNAL as u32);

        let internal_min =
            esp_idf_sys::heap_caps_get_minimum_free_size(esp_idf_sys::MALLOC_CAP_INTERNAL as u32);

        let dma_free = esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_DMA as u32);

        let dma_min =
            esp_idf_sys::heap_caps_get_minimum_free_size(esp_idf_sys::MALLOC_CAP_DMA as u32);

        let psram_free =
            esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_SPIRAM as u32);

        let psram_min =
            esp_idf_sys::heap_caps_get_minimum_free_size(esp_idf_sys::MALLOC_CAP_SPIRAM as u32);

        println!();
        println!("========== MEMORY ==========");

        println!(
            "Internal RAM : {:>6} KiB free | {:>6} KiB low",
            internal_free / 1024,
            internal_min / 1024
        );

        println!(
            "  └─ DMA     : {:>6} KiB free | {:>6} KiB low",
            dma_free / 1024,
            dma_min / 1024
        );

        println!(
            "PSRAM        : {:>6} KiB free | {:>6} KiB low",
            psram_free / 1024,
            psram_min / 1024
        );

        println!("-----------------------------");

        println!(
            "Approx total : {:>6} KiB",
            (internal_free + psram_free) / 1024
        );

        println!();
    }
}

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

    print_memory_stats();
    let log_receiver = init_logging();
    info!("starting");
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    if reason == 9 {
        // OnBoardLed::set_color(RGB8 { r: 9, g: 0, b: 255 });
        thread::sleep(Duration::from_secs(1));
    }

    // OnBoardLed::set_color(RGB8 { r: 15, g: 0, b: 0 });

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

    let ready_pin = PinDriver::input(peripherals.pins.gpio41, Pull::Down)?;
    let master_ready_pin = PinDriver::output(peripherals.pins.gpio39)?;

    let (sender_tx, receiver_tx) = flume::bounded(1000);

    // sender_tx.clo

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
                            sender_tx.send(TheBeginnerCarOutgoingMessages::Log { log }).unwrap();
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
                                                log: LogMessage {
                                                    level: match message.level {
                                                        the_beginner_car_spi_protocol::LogLevel::Error => LogLevel::Error,
                                                        the_beginner_car_spi_protocol::LogLevel::Warn => LogLevel::Warn,
                                                        the_beginner_car_spi_protocol::LogLevel::Info => LogLevel::Info,
                                                        the_beginner_car_spi_protocol::LogLevel::Debug => LogLevel::Debug,
                                                        the_beginner_car_spi_protocol::LogLevel::Trace => LogLevel::Trace,
                                                    },
                                                    target: message.target,
                                                    message: message.message,
                                                    trace_id: message.trace_id,
                                                    system_origin: LogSystemOrigin::WifiMcu,
                                                },
                                            })
                                            .unwrap()
                                    } // TODO: this should send vec
                                    the_beginner_car_spi_protocol::Event::InfraredLedReadings { utmost_front_utmost_left, utmost_front_utmost_right } => {
                                        // dbg!(value);
                                        sender_tx
                                            .send(TheBeginnerCarOutgoingMessages::InfraredLedReadings { utmost_front_utmost_left, utmost_front_utmost_right })
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
                let mut motor_battery = Battery::new(&adc, peripherals.pins.gpio10).unwrap();
                let mut mcu_battery = Battery::new(&adc, peripherals.pins.gpio9).unwrap();

                loop {
                    let motor_battery_reading = motor_battery.read().unwrap();
                    let mcu_battery_reading = mcu_battery.read().unwrap();

                    {
                        sender_tx
                            .send(TheBeginnerCarOutgoingMessages::BatteryReading {
                                motor_battery_voltage: motor_battery_reading.voltage,
                                mcu_battery_voltage: mcu_battery_reading.voltage,
                            })
                            .unwrap();
                    }

                    // dbg!(motor_battery_reading);
                    // dbg!(mcu_battery_reading);
                    let (sender, receiver) = std::sync::mpsc::sync_channel::<
                        Result<ResponseFromHardware, anyhow::Error>,
                    >(1);

                    hardware_sender
                        .send(HardwareMessage {
                            request: RequestToHardware::NotifyBatteryReadings {
                                motor_battery: motor_battery_reading.voltage,
                                mcu_battery: mcu_battery_reading.voltage,
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
        .stack_size(15 * 1024)
        .spawn(move || {
            let mut spi = SpiMaster::new(
                peripherals.spi3,
                peripherals.pins.gpio1.into(),
                peripherals.pins.gpio2.into(),
                peripherals.pins.gpio42.into(),
                ready_pin.into(),
                peripherals.pins.gpio40.into(),
                master_ready_pin,
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
