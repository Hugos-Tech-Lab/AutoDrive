use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use edge_http::io::server::Server;
use esp_idf_svc::hal::{
    gpio::Pull,
    i2c::{I2c, I2cSlaveDriver},
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio::{AnyIOPin, PinDriver},
        i2c::I2cSlaveConfig,
        ledc::{LedcDriver, LedcTimerDriver, config::TimerConfig},
        spi::{Dma, SpiBusDriver, SpiConfig, SpiDriver, SpiDriverConfig},
        units::Hertz,
    },
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
    spi_slave::SpiSlave,
    utils::{heap, stack},
};

use esp_idf_sys::{
    CONFIG_ESP_EFUSE_BLOCK_REV_MAX_FULL, CONFIG_ESP_EFUSE_BLOCK_REV_MIN_FULL, GPIO_PIN18_CONFIG, esp_reset_reason, esp_reset_reason_t_ESP_RST_BROWNOUT, esp_wifi_set_max_tx_power, spi_bus_config_t, spi_bus_config_t__bindgen_ty_1, spi_bus_config_t__bindgen_ty_2, spi_common_dma_t_SPI_DMA_CH_AUTO, spi_dma_chan_t, spi_host_device_t_SPI1_HOST, spi_host_device_t_SPI2_HOST, spi_slave_interface_config_t,
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
use anyhow::Context;
use esp_idf_svc::hal::delay::BLOCK;
use esp_idf_svc::sys::spi_host_device_t;

pub mod spi_master;
pub mod spi_packet;
pub mod spi_slave;

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
    let mut ready_pin = PinDriver::output(peripherals.pins.gpio9)?;

    if reason == 9 {
        // OnBoardLed::set_color(RGB8 { r: 9, g: 0, b: 255 });
        thread::sleep(Duration::from_secs(1));
    }

    // OnBoardLed::set_color(RGB8 { r: 15, g: 0, b: 0 });

    // let auto_script = Arc::new(AutoScript::new()?);

    info!("1: {:?}", heap());

    let thread_receive_incoming_messages =
        std::thread::Builder::new()
            .stack_size(12_000)
            .spawn(move || {
                //                 let bus_config = spi_bus_config_t {
                //     mosi_io_num: GPIO_MOSI,
                //     miso_io_num: GPIO_MISO,
                //     sclk_io_num: GPIO_SCLK,
                //     // ...
                // };

                // let slave_config = spi_slave_interface_config_t {
                //     spics_io_num: GPIO_CS,
                //     // ...
                // };

                let bus_config = spi_bus_config_t {
                    __bindgen_anon_1: spi_bus_config_t__bindgen_ty_1 {
                        data0_io_num: 18, // SDI (slave) - MOSI (master)
                    },
                    __bindgen_anon_2: spi_bus_config_t__bindgen_ty_2 {
                        data1_io_num: 19,
                    },
                    sclk_io_num: 20,
                    max_transfer_sz: 256,
                    ..Default::default()
                };

                let slave_config = spi_slave_interface_config_t {
                    spics_io_num: 21, // CS
                    queue_size: 1,
                    mode: 0,
                    flags: 0,
                    post_setup_cb: None,
                    post_trans_cb: None,
                    ..Default::default()
                };

                let spi = SpiSlave::new(
                    spi_host_device_t_SPI2_HOST,
                    bus_config,
                    slave_config,
                    spi_common_dma_t_SPI_DMA_CH_AUTO,
                )
                .unwrap();
                spi.handle_requests(&mut ready_pin);
            })?;

    thread_receive_incoming_messages.join().unwrap();

    Ok(())
}
