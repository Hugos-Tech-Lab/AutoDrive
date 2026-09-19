use std::{
    sync::{Arc, Mutex},
};

use edge_http::io::server::{Server};
use edge_nal::TcpBind;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio::AnyIOPin,
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
     connect_to_wifi::connect_to_wifi, hardware::on_board_led::OnBoardLed, http_server::{SmallServer, verify_and_set_valid::verify_and_set_valid}, logger::init_logging, auto_script::{AutoScript},
};
use esp_idf_sys::{CONFIG_ESP_EFUSE_BLOCK_REV_MAX_FULL, CONFIG_ESP_EFUSE_BLOCK_REV_MIN_FULL};
use esp_idf_sys::{ESP_APP_DESC_MAGIC_WORD, esp_app_desc_t};
use log::info;
pub mod autoscript;
pub mod connect_to_wifi;
pub mod esp_app_desc_2;
pub mod http_server;
pub mod inter_thread;
pub mod logger;
pub mod auto_script;
pub mod hardware;

use anyhow::Context;

esp_app_desc_2! {}

pub fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

unsafe {
        let config = esp_idf_sys::esp_vfs_eventfd_config_t {
            max_fds: 5,
        };
        esp_idf_sys::esp_vfs_eventfd_register(&config);
    }
    
    init_logging();
    info!("starting");

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let _hardware = OnBoardLed::new(peripherals.pins.gpio8, peripherals.spi2);
    let auto_script = Arc::new(AutoScript::new());

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;
    connect_to_wifi(&mut wifi)?;

    let mut mdns = EspMdns::take()?;
    mdns.set_hostname("the-beginner")?;

    let mut ota = EspOta::new().context("failed to obtain OTA instance")?;

    verify_and_set_valid(&mut ota)?;
    let esp_ota = Arc::new(Mutex::new(ota));

    // let _device_control = Arc::new(Mutex::new(DeviceControl::new()?));
    // info!("activating auto");

    let handle = std::thread::Builder::new()
        .name("async_main".into())
        .stack_size(48 * 1024)
        .spawn(|| {
            let mut server = SmallServer::new();
            futures_lite::future::block_on(http_server::run(&mut server, auto_script))
        })?;

    handle.join().unwrap().unwrap();
    Ok(())
}
