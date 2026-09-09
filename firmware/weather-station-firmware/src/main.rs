//! Example of using a blocking Wifi with a DHCP configuration that has a user-supplied host name
//!
//! Add your own SSID and password for the access point
//!
//! Once the wifi is connected, the hostname will be set to "foo"
//! Try pinging it from your PC with `ping foo`

#![allow(unknown_lints)]
#![allow(unexpected_cfgs)]

use esp_idf_svc::mdns::EspMdns;

#[cfg(not(any(esp32h2, esp32h4, esp32p4)))]
fn main() -> anyhow::Result<()> {
    example::main()
}

#[cfg(any(esp32h2, esp32h4, esp32p4))]
fn main() -> anyhow::Result<()> {
    panic!(
        "ESP32-H2, ESP32-H4 and ESP32-P4 do not have a Wifi radio (but you could enable the esp-wifi-remote component to use them with a WiFi co-processor)"
    );
}

#[cfg(not(any(esp32h2, esp32h4, esp32p4)))]
mod example {
    use core::convert::TryInto;
    use dotenvy_macro::dotenv;

    use embedded_dht_rs::dht11::Dht11;
    use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration as WifiConfiguration};
    use esp_idf_svc::hal::delay::{Delay, FreeRtos};
    use esp_idf_svc::hal::gpio::PinDriver;
    use esp_idf_svc::hal::gpio::Pull;
    use esp_idf_svc::hal::ledc::config::TimerConfig;
    use esp_idf_svc::hal::ledc::{LedcDriver, LedcTimerDriver};
    use esp_idf_svc::hal::peripherals::Peripherals;
    use esp_idf_svc::hal::units::*;
    use esp_idf_svc::http::Method;
    use esp_idf_svc::http::server::EspHttpServer;
    use esp_idf_svc::io::Write;
    use esp_idf_svc::ipv4::{
        ClientConfiguration as IpClientConfiguration, Configuration as IpConfiguration,
        DHCPClientSettings,
    };
    use esp_idf_svc::log::EspLogger;
    use esp_idf_svc::mdns::EspMdns;
    use esp_idf_svc::netif::{EspNetif, NetifConfiguration, NetifStack};
    use esp_idf_svc::sys::{esp_wifi_set_ps, ledc_get_freq, wifi_ps_type_t_WIFI_PS_NONE};
    use esp_idf_svc::wifi::{BlockingWifi, EspWifi, WifiDriver};
    use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
    use std::sync::Mutex;

    use log::info;

    const SSID: &str = dotenv!("WIFI_NAME");
    const PASSWORD: &str = dotenv!("WIFI_PASSWORD");

    pub fn main() -> anyhow::Result<()> {
        esp_idf_svc::sys::link_patches();
        EspLogger::initialize_default();

        let peripherals = Peripherals::take()?;
        // GPIO21 = direction control
        let mut dir_pin = PinDriver::output(peripherals.pins.gpio9)?;

        // Configure LEDC timer.
        //
        // 20 kHz is a reasonable PWM frequency for a motor driver.
        let timer_driver = LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &TimerConfig::default().frequency(20.kHz().into()),
        )?;

        // GPIO20 = PWM1
        let mut pwm_driver = LedcDriver::new(
            peripherals.ledc.channel0,
            timer_driver,
            peripherals.pins.gpio15,
        )?;

        // Start stopped.
        pwm_driver.set_duty(pwm_driver.get_max_duty())?;

        // 50% duty cycle
        let half_duty = pwm_driver.get_max_duty() / 2;

        loop {
            // ---------------------------------------------------------
            // Direction 1
            // ---------------------------------------------------------
            dir_pin.set_low()?;

            log::info!("Motor direction 1, PWM = 50%");
            pwm_driver.set_duty(half_duty)?;

            // Run for 5 seconds
            FreeRtos::delay_ms(5000);

            // Stop motor
            log::info!("Motor stopped");
            pwm_driver.set_duty(0)?;

            // Wait 1 second before changing direction
            FreeRtos::delay_ms(1000);

            // ---------------------------------------------------------
            // Direction 2
            // ---------------------------------------------------------
            dir_pin.set_high()?;

            log::info!("Motor direction 2, PWM = 50%");
            pwm_driver.set_duty(half_duty)?;

            // Run for 5 seconds
            FreeRtos::delay_ms(5000);

            // Stop motor
            log::info!("Motor stopped");
            pwm_driver.set_duty(0)?;

            // Wait 1 second before changing direction
            FreeRtos::delay_ms(1000);
        }

        Ok(())
    }
}
