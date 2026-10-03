use std::{
    sync::{Arc, Mutex, mpsc::SyncSender},
    thread,
    time::Duration,
};

use common_firmware::{
    memory::{print_heap, print_stack},
    spi_master::SpiMaster,
    the_beginner_car::{RequestToHardware, ResponseFromHardware},
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio::{AnyIOPin, PinDriver, Pull},
        ledc::{LedcDriver, LedcTimerDriver, config::TimerConfig},
        spi::{Dma, SpiBusDriver, SpiConfig, SpiDriver, SpiDriverConfig},
        units::Hertz,
    },
    mdns::EspMdns,
    ota::EspOta,
    wifi::{BlockingWifi, EspWifi},
};
#[cfg(all(esp_idf_app_compile_time_date, not(esp_idf_app_reproducible_build)))]
use esp_idf_svc::{
    hal::peripherals::Peripherals,
    nvs::EspDefaultNvsPartition,
    sys::{build_time::build_time_utc, const_format},
};
use futures::{FutureExt, executor::LocalPool, task::LocalSpawnExt};

use crate::{
    auto_script::AutoScript,
    connect_to_wifi::connect_to_wifi,
    logger::init_logging,
    tcp_server::{accept, verify_and_set_valid::verify_and_set_valid},
    utils::{heap, stack},
};

use esp_idf_sys::{
    CONFIG_ESP_EFUSE_BLOCK_REV_MAX_FULL, CONFIG_ESP_EFUSE_BLOCK_REV_MIN_FULL, esp_reset_reason,
    esp_reset_reason_t_ESP_RST_BROWNOUT, esp_wifi_set_max_tx_power,
};
use esp_idf_sys::{ESP_APP_DESC_MAGIC_WORD, esp_app_desc_t};
use log::info;
pub mod auto_script;
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

        println!("  DMA is a capability of internal RAM,");
        println!("  not additional memory.");
        println!("=============================");
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
    init_logging();
    info!("starting");
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    // let mut hardware = Hardware::new(peripherals.pins.gpio38.into(), peripherals.spi2);

    // hardware.off();

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

    mdns.set_hostname("the-beginner")?;

    info!("setting host name");
    let (sender, receiver) = std::sync::mpsc::sync_channel::<HardwareMessage>(100);

    // let _http_server = tcp_server::run(sender).unwrap();

    let mut ready_pin = PinDriver::input(peripherals.pins.gpio9, Pull::Down)?;
    let mut master_ready_pin = PinDriver::output(peripherals.pins.gpio46)?;

    std::thread::Builder::new()
        .stack_size(40 * 1000)
        .spawn(|| {
            let mut local_executor = LocalPool::new();
            let spawner = local_executor.spawner();

            local_executor
                .spawner()
                .spawn_local(
                    async move {
                        accept(spawner, sender).await.unwrap();

                        Result::<_, anyhow::Error>::Ok(())
                    }
                    .map(Result::unwrap),
                )
                .unwrap();

            local_executor.run();
        })
        .unwrap()
        .join()
        .unwrap();

    let handle = std::thread::Builder::new()
        .name("i2c_driver".into())
        .stack_size(15 * 1024)
        .spawn(move || {
            let mut spi = SpiMaster::new(
                peripherals.spi3,
                peripherals.pins.gpio12.into(),
                peripherals.pins.gpio10.into(),
                peripherals.pins.gpio11.into(),
                ready_pin.into(),
                peripherals.pins.gpio13.into(),
                master_ready_pin,
                1_000_000,
            )
            .unwrap();

            print_heap();
            print_stack();

            loop {
                match receiver.recv() {
                    Ok(message) => {
                        println!("{:?}", message.request);

                        let res: Result<ResponseFromHardware, anyhow::Error> =
                            spi.send_request(&message.request);

                        info!("RESPONSE: {:?}", res);

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

    print_memory_stats();
    handle.join().unwrap();

    // handle.join().unwrap().unwrap();
    Ok(())
}
