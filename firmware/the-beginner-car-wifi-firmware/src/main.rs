use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use edge_http::io::server::Server;
use edge_nal::TcpBind;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio::{AnyIOPin, PinDriver},
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

use crate::{
    auto_script::AutoScript,
    connect_to_wifi::connect_to_wifi,
    http_server::{SmallServer, verify_and_set_valid::verify_and_set_valid},
    logger::init_logging,
    utils::{heap, stack},
};

use esp_idf_sys::{CONFIG_ESP_EFUSE_BLOCK_REV_MAX_FULL, CONFIG_ESP_EFUSE_BLOCK_REV_MIN_FULL, esp_reset_reason, esp_reset_reason_t_ESP_RST_BROWNOUT, esp_wifi_set_max_tx_power};
use esp_idf_sys::{ESP_APP_DESC_MAGIC_WORD, esp_app_desc_t};
use log::info;
use smart_leds_trait::RGB8;
pub mod auto_script;
pub mod connect_to_wifi;
pub mod esp_app_desc_2;
pub mod hardware;
pub mod http_server;
pub mod inter_thread;
pub mod logger;
pub mod utils;

use anyhow::Context;

pub fn print_memory_stats() {
    unsafe {
        let internal_free =
            esp_idf_sys::heap_caps_get_free_size(
                esp_idf_sys::MALLOC_CAP_INTERNAL as u32,
            );

        let internal_min =
            esp_idf_sys::heap_caps_get_minimum_free_size(
                esp_idf_sys::MALLOC_CAP_INTERNAL as u32,
            );

        let dma_free =
            esp_idf_sys::heap_caps_get_free_size(
                esp_idf_sys::MALLOC_CAP_DMA as u32,
            );

        let dma_min =
            esp_idf_sys::heap_caps_get_minimum_free_size(
                esp_idf_sys::MALLOC_CAP_DMA as u32,
            );

        let psram_free =
            esp_idf_sys::heap_caps_get_free_size(
                esp_idf_sys::MALLOC_CAP_SPIRAM as u32,
            );

        let psram_min =
            esp_idf_sys::heap_caps_get_minimum_free_size(
                esp_idf_sys::MALLOC_CAP_SPIRAM as u32,
            );

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

pub fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    unsafe {
        let config = esp_idf_sys::esp_vfs_eventfd_config_t {
            max_fds: 16,
        };

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
    // let _hardware = HardwareState::new(
    //     peripherals.pins.gpio38,
    //     peripherals.spi2,
    //     peripherals.pins.gpio9,
    //     peripherals.pins.gpio10,
    //     peripherals.pins.gpio11,
    //     peripherals.pins.gpio12,
    //     peripherals.ledc.timer0,
    //     peripherals.ledc.timer1,
    //     peripherals.ledc.channel0,
    //     peripherals.ledc.channel1,
    // );

    if reason == 9 {
        // OnBoardLed::set_color(RGB8 { r: 9, g: 0, b: 255 });
        thread::sleep(Duration::from_secs(1));
    }


    // OnBoardLed::set_color(RGB8 { r: 15, g: 0, b: 0 });

    let auto_script = Arc::new(AutoScript::new()?);
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    info!("123123123: {:?}", heap());

    connect_to_wifi(&mut wifi)?;
    info!("2333333333333: {:?}", heap());


    let mut mdns = EspMdns::take()?;
        info!("3434334343434: {:?}", heap());

    mdns.set_hostname("the-beginner")?;

    let handle = std::thread::Builder::new()
        .name("http_server".into())
        .stack_size(50 * 1024)
        .spawn(|| {
            let mut server = SmallServer::new();
            futures_lite::future::block_on(http_server::run(&mut server))
        })?;
    // OnBoardLed::set_color(RGB8 {
    //     r: 15,
    //     g: 15,
    //     b: 15,
    // });

    print_memory_stats();
    handle.join().unwrap().unwrap();
    loop {
        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}
