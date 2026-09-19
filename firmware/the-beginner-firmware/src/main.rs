use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use edge_http::io::server::{DefaultServer, Server};
use edge_nal::TcpBind;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio::AnyIOPin,
        spi::{Dma, SpiBusDriver, SpiConfig, SpiDriver, SpiDriverConfig},
        units::Hertz,
    },
    http::{
        Method,
        server::{Configuration, EspHttpServer},
    },
    io::Write,
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
    autoscript::AutoScript, autoscript_v2::HttpHandler, connect_to_wifi::connect_to_wifi, device_control::DeviceControl, hardware::on_board_led::OnBoardLed, http::verify_and_set_valid::verify_and_set_valid, logger::init_logging, wasm::Wasm,
};
use esp_idf_sys::{CONFIG_ESP_EFUSE_BLOCK_REV_MAX_FULL, CONFIG_ESP_EFUSE_BLOCK_REV_MIN_FULL};
use esp_idf_sys::{ESP_APP_DESC_MAGIC_WORD, esp_app_desc_t};
use log::info;
pub mod autoscript;
pub mod autoscript_v2;
pub mod connect_to_wifi;
pub mod device_control;
pub mod esp_app_desc_2;
pub mod hardware;
pub mod http;
pub mod inter_thread;
pub mod logger;
pub mod wasm;

use anyhow::Context;

pub type SmallServer = Server<2, 1024, 16>;

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
    let wasm = Wasm::new();
    let auto_script = Arc::new(AutoScript::new(wasm));

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;
    connect_to_wifi(&mut wifi)?;

    // let server_config = Configuration {
    //     uri_match_wildcard: true,
    //     stack_size: 16 * 1024,
    //     max_open_sockets: 7,
    //     ..Default::default()
    // };
    // let mut server = EspHttpServer::new(&server_config)?;

    let mut mdns = EspMdns::take()?;
    mdns.set_hostname("the-beginner")?;

    let mut ota = EspOta::new().context("failed to obtain OTA instance")?;

    verify_and_set_valid(&mut ota)?;
    let esp_ota = Arc::new(Mutex::new(ota));

    // let _device_control = Arc::new(Mutex::new(DeviceControl::new()?));
    info!("activating auto");
    let device_control = DeviceControl::new().unwrap();
    device_control.activate_auto();


    // Keep main's stack frame tiny (< 250 bytes)
    let handle = std::thread::Builder::new()
        .name("async_main".into())
        .stack_size(74 * 1024)
        .spawn(|| {
            // Instantiate DefaultServer inside the spawned thread with 32KB stack
            let mut server = SmallServer::new();
            futures_lite::future::block_on(run(&mut server, auto_script))
        })?;

    handle.join().unwrap().unwrap();
    Ok(())
}

pub async fn run(server: &mut SmallServer, auto_script: Arc<AutoScript>) -> Result<(), anyhow::Error> {

    let addr ="0.0.0.0:80".parse().unwrap();
    info!("Running HTTP server on {addr}");

    let acceptor = edge_nal_std::Stack::new()
        .bind(addr)
        .await?;

    server.run(None, acceptor, HttpHandler { auto_script }).await?;

    Ok(())
}
