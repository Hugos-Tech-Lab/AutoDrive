use dotenvy_macro::dotenv;
use esp_idf_svc::{wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi}};

use esp_idf_sys::{esp_wifi_set_ps, wifi_ps_type_t_WIFI_PS_NONE};
use log::info;

const SSID: &str = dotenv!("WIFI_NAME");
const PASSWORD: &str = dotenv!("WIFI_PASSWORD"); 

pub fn connect_to_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connected");

    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("Wifi Interface info: {ip_info:?}");

    unsafe { esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_NONE); }

    Ok(())
}