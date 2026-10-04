use std::{
    sync::{Arc, Mutex}, thread, time::{Duration, Instant},
};

use common_firmware::{
    get_mac_address::{format_mac_address, get_mac_address},
    on_board_led::OnBoardLED,
    spi_slave::SpiSlave,
    the_beginner_car::{RequestToHardware, ResponseFromHardware},
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::gpio::PinDriver};
#[cfg(all(esp_idf_app_compile_time_date, not(esp_idf_app_reproducible_build)))]
use esp_idf_svc::{
    hal::peripherals::Peripherals,
    nvs::EspDefaultNvsPartition,
    sys::{build_time::build_time_utc, const_format},
};

use crate::{
    auto_script::AutoScript,
    hardware::{TheBeginnerCarHardware, motor::Motor},
    logger::init_logging,
};

use esp_idf_sys::{
    CONFIG_ESP_EFUSE_BLOCK_REV_MAX_FULL, CONFIG_ESP_EFUSE_BLOCK_REV_MIN_FULL, GPIO_PIN18_CONFIG,
    esp_reset_reason, esp_reset_reason_t_ESP_RST_BROWNOUT, esp_wifi_set_max_tx_power,
    spi_bus_config_t, spi_bus_config_t__bindgen_ty_1, spi_bus_config_t__bindgen_ty_2,
    spi_common_dma_t_SPI_DMA_CH_AUTO, spi_dma_chan_t, spi_host_device_t_SPI1_HOST,
    spi_host_device_t_SPI2_HOST, spi_slave_interface_config_t,
};
use esp_idf_sys::{ESP_APP_DESC_MAGIC_WORD, esp_app_desc_t};
use log::info;
use smart_leds_trait::RGB8;
pub mod auto_script;
pub mod esp_app_desc_2;
pub mod hardware;
pub mod inter_thread;
pub mod lib;
pub mod logger;

pub fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    unsafe {
        let config = esp_idf_sys::esp_vfs_eventfd_config_t { max_fds: 5 };
        esp_idf_sys::esp_vfs_eventfd_register(&config);
    }

    let reason = unsafe { esp_reset_reason() };
    println!("Last reset reason: {:?}", reason);

    let addr = format_mac_address(&get_mac_address()?);
    dbg!(addr);

    let log_message_receiver = init_logging();

    info!("starting");
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let mut motor_a = Arc::new(Mutex::new(Motor::new(
        peripherals.pins.gpio0.into(),
        peripherals.pins.gpio7.into(),
        peripherals.ledc.channel0,
        peripherals.ledc.timer1,
    )));
    let mut motor_b = Arc::new(Mutex::new(Motor::new(
        peripherals.pins.gpio5.into(),
        peripherals.pins.gpio6.into(),
        peripherals.ledc.channel1,
        peripherals.ledc.timer2,
    )));
    // let hardware = TheBeginnerCarHardware::

    // let pin = OnBoardLED::new(peripherals.pins.gpio8.into(), peripherals.spi2);

    // pin.set_color(20, 2, 2);
    if reason == 9 {
        // pin.set_color(9, 0, 200);
        thread::sleep(Duration::from_secs(1));
    }

    // let auto_script = Arc::new(AutoScript::new()?);

    // info!("1: {:?}", heap());

    let thread_receive_incoming_messages =
        std::thread::Builder::new()
            .stack_size(12_000)
            .spawn(move || {
                let mut spi = SpiSlave::new(
                    peripherals.spi2,
                    peripherals.pins.gpio20,
                    peripherals.pins.gpio18,
                    peripherals.pins.gpio19,
                    peripherals.pins.gpio9,
                    peripherals.pins.gpio21,
                    peripherals.pins.gpio22,
                )
                .unwrap();

                // pin.set_color(3, 20, 3);
                spi.listen(|request: RequestToHardware| -> ResponseFromHardware {
                    match request {
                        RequestToHardware::LightOn => println!("LightOn"),
                        RequestToHardware::LightOff => println!("LightOff"),
                        RequestToHardware::SetMotorASpeed(speed) => {
                            let mut motor_a = motor_a.lock().unwrap();
                            motor_a.set_speed(speed).unwrap();

                        }
                        RequestToHardware::SetMotorBSpeed(speed) => {
                            let mut motor_b = motor_b.lock().unwrap();
                            motor_b.set_speed(speed).unwrap();
                        }
                        RequestToHardware::Logs => println!("Logs"),
                    }

                    // pin.set_color(3, 20, 3);
                    ResponseFromHardware::Ok
                });
            })?;

    thread_receive_incoming_messages.join().unwrap();

    Ok(())
}
