use std::{
    sync::{Arc, Mutex}, thread, time::Duration,
};

use common_firmware::{
    get_mac_address::{format_mac_address, get_mac_address},
    spi_slave::SpiSlave,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::{adc::{AdcContDriver, attenuation::DB_12, oneshot::{AdcChannelDriver, AdcDriver, config::AdcChannelConfig}}, gpio::Pin}};
#[cfg(all(esp_idf_app_compile_time_date, not(esp_idf_app_reproducible_build)))]
use esp_idf_svc::{
    hal::peripherals::Peripherals,
    nvs::EspDefaultNvsPartition
};
use the_beginner_car_spi_protocol::{RequestToHardware, ResponseFromHardware};

use crate::{
    hardware::motor::Motor, logger::init_logging,
};

use esp_idf_sys::{
    CONFIG_ESP_EFUSE_BLOCK_REV_MAX_FULL, CONFIG_ESP_EFUSE_BLOCK_REV_MIN_FULL, GPIO_PIN18_CONFIG,
    esp_reset_reason, esp_reset_reason_t_ESP_RST_BROWNOUT, esp_wifi_set_max_tx_power,
    spi_bus_config_t, spi_bus_config_t__bindgen_ty_1, spi_bus_config_t__bindgen_ty_2,
    spi_common_dma_t_SPI_DMA_CH_AUTO, spi_dma_chan_t, spi_host_device_t_SPI1_HOST,
    spi_host_device_t_SPI2_HOST, spi_slave_interface_config_t,
};
use log::info;
pub mod auto_script;
pub mod esp_app_desc_2;
pub mod hardware;
pub mod inter_thread;
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
    let motor_a = Arc::new(Mutex::new(Motor::new(
        peripherals.pins.gpio15.into(),
        peripherals.pins.gpio23.into(),
        peripherals.ledc.channel0,
        peripherals.ledc.timer1,
    )));
    let motor_b = Arc::new(Mutex::new(Motor::new(
        peripherals.pins.gpio21.into(),
        peripherals.pins.gpio22.into(),
        peripherals.ledc.channel1,
        peripherals.ledc.timer2,
    )));

    // let pin = OnBoardLED::new(peripherals.pins.gpio8.into(), peripherals.spi2);

    // pin.set_color(20, 2, 2);
    if reason == 9 {
        // pin.set_color(9, 0, 200);
        thread::sleep(Duration::from_secs(1));
    }

    let adc = AdcDriver::new(peripherals.adc1)?;
    // Configure ADC input attenuation
    let config = AdcChannelConfig {
        attenuation: DB_12,
        ..Default::default()
    };

    // let aaa = ;
    // dbg!(aaa.pin());

    let mut pin =
        AdcChannelDriver::new(&adc, peripherals.pins.gpio3, &config)?;

    // loop {
    //     let value = adc.read(&mut pin)?;

    //     println!("TCRT5000 ADC: {}", value);

    //     thread::sleep(Duration::from_millis(100));
    // }


    // let auto_script = Arc::new(AutoScript::new()?);

    // info!("1: {:?}", heap());

    let thread_receive_incoming_messages =
        std::thread::Builder::new()
            .stack_size(12_000)
            .spawn(move || {
                let mut spi = SpiSlave::new(
                    peripherals.spi2,
                    peripherals.pins.gpio4,
                    peripherals.pins.gpio5,
                    peripherals.pins.gpio6,
                    peripherals.pins.gpio7,
                    peripherals.pins.gpio0,
                    peripherals.pins.gpio1,
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
                        RequestToHardware::NotifyPositionFromCamera { x, y } => println!("pos from camera"),
                        RequestToHardware::NotifyBatteryReadings { motor_battery, mcu_battery }  => println!("motor battery from mcu"),
                    }

                    // pin.set_color(3, 20, 3);
                    ResponseFromHardware::Ok
                });
            })?;

    thread_receive_incoming_messages.join().unwrap();

    Ok(())
}
