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
    panic!("ESP32-H2, ESP32-H4 and ESP32-P4 do not have a Wifi radio (but you could enable the esp-wifi-remote component to use them with a WiFi co-processor)");
}

#[cfg(not(any(esp32h2, esp32h4, esp32p4)))]
mod example {
    use core::convert::TryInto;
use dotenvy_macro::dotenv;

    use embedded_dht_rs::dht11::Dht11;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration as WifiConfiguration};

    use esp_idf_svc::hal::delay::{Delay, FreeRtos};
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
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
    use esp_idf_svc::sys::{esp_wifi_set_ps, wifi_ps_type_t_WIFI_PS_NONE};
    use esp_idf_svc::wifi::{BlockingWifi, EspWifi, WifiDriver};
    use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
    use esp_idf_svc::hal::gpio::Pull;

    use log::info;

    const SSID: &str = dotenv!("WIFI_NAME");
    const PASSWORD: &str = dotenv!("WIFI_PASSWORD"); 

    pub fn main() -> anyhow::Result<()> {
        esp_idf_svc::sys::link_patches();
        EspLogger::initialize_default();

        let peripherals = Peripherals::take()?;
        let sys_loop = EspSystemEventLoop::take()?;
        let nvs = EspDefaultNvsPartition::take()?;

        let dht11_pin = PinDriver::input_output_od(peripherals.pins.gpio4, Pull::Floating)?;

        // let dht11_pin =
            // PinDriver::output_od(peripherals.pins.gpio4)?;``
        let delay = Delay::new_default();

        let mut dht11 = Dht11::new(dht11_pin, delay);


    loop {
        log::info!("starting");
        FreeRtos::delay_ms(2000);
        log::info!("done delay");

        match dht11.read() {
            Ok(sensor_reading) => log::info!(
                "DHT 11 Sensor - Temperature: {} °C, humidity: {} %",
                sensor_reading.temperature,
                sensor_reading.humidity
            ),
            Err(error) => log::error!("An error occurred while trying to read sensor: {:?}", error),
        }
    }
        // let wifi = WifiDriver::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;
        // let wifi = configure_wifi(wifi)?;

        // let mut wifi = BlockingWifi::wrap(wifi, sys_loop)?;
        // connect_wifi(&mut wifi)?;

        // let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
        // let mut mdns = EspMdns::take()?;
        // mdns.set_hostname("esp-advertiser")?;
        
        // let server_config = esp_idf_svc::http::server::Configuration::default();
        // let mut server = EspHttpServer::new(&server_config)?;
        // server.fn_handler("/", Method::Get, |req| {
        //     req.into_ok_response()?
        //         .write_all("hi".as_bytes())
        //         .map(|_| ())
        // })?;

        // // Advertise the HTTP server
        // // mdns.add_service(
        // //     Some("ESP HTTP Server"),
        // //     "_http",
        // //     "_tcp",
        // //     server_config.http_port,
        // //     &[],
        // // )?;

        // info!("Wifi Interface info: {ip_info:?}");

        // unsafe { esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_NONE); }
        // loop {
        //     std::thread::sleep(core::time::Duration::from_secs(5));
        // }

        // let od_for_dht11 = OutputOpenDrain::new(io.pins.gpio4, Level::High, Pull::None);


    }

    fn configure_wifi(wifi: WifiDriver) -> anyhow::Result<EspWifi> {
        let mut wifi = EspWifi::wrap_all(
            wifi,
            // Note that setting a custom hostname can be used with any network adapter, not just Wifi
            // I.e. that would work with Eth as well, because DHCP is an L3 protocol
            EspNetif::new_with_conf(&NetifConfiguration {
                ip_configuration: Some(IpConfiguration::Client(IpClientConfiguration::DHCP(
                    DHCPClientSettings {
                        hostname: Some("foo".try_into().unwrap()),
                    },
                ))),
                ..NetifConfiguration::wifi_default_client()
            })?,
            #[cfg(esp_idf_esp_wifi_softap_support)]
            EspNetif::new(NetifStack::Ap)?,
        )?;

        let wifi_configuration = WifiConfiguration::Client(ClientConfiguration {
            ssid: SSID.try_into().unwrap(),
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: PASSWORD.try_into().unwrap(),
            channel: None,
            ..Default::default()
        });
        wifi.set_configuration(&wifi_configuration)?;

        Ok(wifi)
    }

    fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
        wifi.start()?;
        info!("Wifi started");

        wifi.connect()?;
        info!("Wifi connected");

        wifi.wait_netif_up()?;
        info!("Wifi netif up");

        Ok(())
    }
}
