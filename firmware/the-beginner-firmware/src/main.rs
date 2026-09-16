use std::{sync::{Arc, Mutex}, thread, time::Duration};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop, hal::{gpio::AnyIOPin, spi::{Dma, SpiBusDriver, SpiConfig, SpiDriver, SpiDriverConfig}, units::Hertz}, http::{
        Method,
        server::{Configuration, EspHttpServer},
    }, io::Write, mdns::EspMdns, ota::EspOta, wifi::{BlockingWifi, EspWifi},
};
#[cfg(all(esp_idf_app_compile_time_date, not(esp_idf_app_reproducible_build)))]
use esp_idf_svc::{
    hal::peripherals::Peripherals,
    nvs::EspDefaultNvsPartition,
    sys::{build_time::build_time_utc, const_format},
};

use crate::{
    connect_to_wifi::connect_to_wifi, device_control::DeviceControl, hardware::{on_board_led::OnBoardLed}, logger::init_logging, http::verify_and_set_valid::verify_and_set_valid,
};
use esp_idf_sys::{CONFIG_ESP_EFUSE_BLOCK_REV_MAX_FULL, CONFIG_ESP_EFUSE_BLOCK_REV_MIN_FULL};
use esp_idf_sys::{ESP_APP_DESC_MAGIC_WORD, esp_app_desc_t};
use log::info;
pub mod wasm;
pub mod connect_to_wifi;
pub mod esp_app_desc_2;
pub mod logger;
pub mod http;
pub mod hardware;
pub mod device_control;

use anyhow::Context;

esp_app_desc_2! {}

pub fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    init_logging();
    info!("starting");

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let _hardware = OnBoardLed::new(peripherals.pins.gpio8, peripherals.spi2);

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;
    connect_to_wifi(&mut wifi)?;

    let server_config = Configuration {
        uri_match_wildcard: true,
        stack_size: 16 * 1024,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&server_config)?;

    let mut mdns = EspMdns::take()?;
    mdns.set_hostname("the-beginner")?;

    let mut ota = EspOta::new().context("failed to obtain OTA instance")?;

    verify_and_set_valid(&mut ota)?;
    let esp_ota = Arc::new(Mutex::new(ota));

    // let _device_control = Arc::new(Mutex::new(DeviceControl::new()?));
    info!("activating auto");
    let device_control = DeviceControl::new().unwrap();
    device_control.activate_auto();

    info!("registering");
    http::set_handles(&mut server, esp_ota)?;

    server.fn_handler("/*", Method::Get, |req| -> anyhow::Result<()> {
        req.into_status_response(404)?.write_all(b"Not Found")?;
        Ok(())
    })?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
