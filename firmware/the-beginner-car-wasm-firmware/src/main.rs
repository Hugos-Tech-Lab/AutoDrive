use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use edge_http::io::server::Server;
use esp_idf_svc::hal::i2c::{I2c, I2cSlaveDriver};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio::{AnyIOPin, PinDriver},
        i2c::I2cSlaveConfig,
        ledc::{LedcDriver, LedcTimerDriver, config::TimerConfig},
        spi::{Dma, SpiBusDriver, SpiConfig, SpiDriver, SpiDriverConfig},
        units::Hertz,
    }
};
#[cfg(all(esp_idf_app_compile_time_date, not(esp_idf_app_reproducible_build)))]
use esp_idf_svc::{
    hal::peripherals::Peripherals,
    nvs::EspDefaultNvsPartition,
    sys::{build_time::build_time_utc, const_format},
};

use crate::{
    auto_script::AutoScript,
    hardware::on_board_led::OnBoardLed,
    logger::init_logging,
    utils::{heap, stack},
};

use esp_idf_sys::{
    CONFIG_ESP_EFUSE_BLOCK_REV_MAX_FULL, CONFIG_ESP_EFUSE_BLOCK_REV_MIN_FULL, esp_reset_reason,
    esp_reset_reason_t_ESP_RST_BROWNOUT, esp_wifi_set_max_tx_power,
};
use esp_idf_sys::{ESP_APP_DESC_MAGIC_WORD, esp_app_desc_t};
use log::info;
use smart_leds_trait::RGB8;
pub mod auto_script;
pub mod esp_app_desc_2;
pub mod hardware;
pub mod inter_thread;
pub mod logger;
pub mod utils;
use esp_idf_svc::hal::delay::BLOCK;

use anyhow::Context;


pub mod incoming_messages;
pub mod outgoing_messages;

pub fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    unsafe {
        let config = esp_idf_sys::esp_vfs_eventfd_config_t { max_fds: 5 };
        esp_idf_sys::esp_vfs_eventfd_register(&config);
    }

    let reason = unsafe { esp_reset_reason() };
    println!("Last reset reason: {:?}", reason);

    let log_message_receiver = init_logging();


    info!("starting");
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    // let _hardware = OnBoardLed::new(
    //     peripherals.pins.gpio,
    //     peripherals.spi2,
    //     peripherals.pins.gpio15,
    //     peripherals.pins.gpio21,
    //     peripherals.pins.gpio22,
    //     peripherals.pins.gpio23,
    //     peripherals.ledc.timer0,
    //     peripherals.ledc.timer1,
    //     peripherals.ledc.channel0,
    //     peripherals.ledc.channel1,
    // );

    if reason == 9 {
        OnBoardLed::set_color(RGB8 { r: 9, g: 0, b: 255 });
        thread::sleep(Duration::from_secs(1));
    }

    OnBoardLed::set_color(RGB8 { r: 15, g: 0, b: 0 });

    // let auto_script = Arc::new(AutoScript::new()?);

    info!("1: {:?}", heap());

    OnBoardLed::set_color(RGB8 {
        r: 15,
        g: 15,
        b: 15,
    });

    let thread0 = std::thread::Builder::new()
        .stack_size(7000)
        .spawn(move || {
            incoming_messages::receive_loop(peripherals).unwrap();
        })?;

    Ok(())
}
