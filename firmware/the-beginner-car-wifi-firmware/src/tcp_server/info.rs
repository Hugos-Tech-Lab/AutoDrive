use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use esp_idf_svc::{
    http::{Method, server::EspHttpServer},
    ota::{EspOta, SlotState},
    sys,
};

use serde::Serialize;

#[derive(Serialize)]
struct Device {
    id: String,
    esp_idf_version: String,
    mac: String,
    firmware_version: String,
    firmware_build_time_utc: String,
    firmware_description: String,
    uptime_seconds: i64,
    running_slot_label: String,
    running_slot_state: SlotState,
    #[serde(rename = "freeHeap")]
    free_heap: i64,
}

pub fn set_handles(server: &mut EspHttpServer, esp_ota: Arc<Mutex<EspOta>>) -> anyhow::Result<()> {
    server.fn_handler("/firmware/info", Method::Options, |req| -> anyhow::Result<()> {
        req.into_response(
            204,
            None,
            &[
                ("Access-Control-Allow-Origin", "*"),
                ("Access-Control-Allow-Methods", "GET, OPTIONS"),
                ("Access-Control-Allow-Headers", "Content-Type"),
            ],
        )?;
        Ok(())
    })?;
    server.fn_handler("/firmware/info", Method::Get, move |req| -> anyhow::Result<()> {
        let esp_ota = esp_ota.lock().unwrap();
        let device = get_device_info(&esp_ota)?;

        let body = serde_json::to_vec(&device)?;

        let mut response = req.into_response(
            200,
            None,
            &[
                ("Content-Type", "application/json"),
                ("Access-Control-Allow-Origin", "*"),
            ],
        )?;

        response.write(&body)?;

        Ok(())
    })?;

    Ok(())
}

fn get_device_info(ota: &EspOta) -> anyhow::Result<Device> {
    let mut mac = [0u8; 6];

    unsafe {
        sys::esp_read_mac(mac.as_mut_ptr(), sys::esp_mac_type_t_ESP_MAC_WIFI_STA);
    }

    let mac_string = mac
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(":");

    let id = format!("esp32-{:02X}{:02X}", mac[4], mac[5]);

    let esp_idf_version = unsafe { std::ffi::CStr::from_ptr(sys::esp_get_idf_version()) }
        .to_string_lossy()
        .into_owned();

    let uptime_seconds = unsafe { sys::esp_timer_get_time() / 1_000_000 };

    let free_heap = unsafe { sys::esp_get_free_heap_size() };

    let slot = ota.get_running_slot()?;

    let firmware_info = slot
        .firmware
        .ok_or(anyhow!("missing firmware info for running slot"))?;

    Ok(Device {
        id,
        esp_idf_version,
        mac: mac_string,
        firmware_version: firmware_info.version.to_string(),
        firmware_build_time_utc: firmware_info.released.to_string(),
        firmware_description: firmware_info.description.unwrap_or_default().to_string(),
        running_slot_label: slot.label.to_string(),
        running_slot_state: slot.state,
        uptime_seconds,
        free_heap: free_heap as i64,
    })
}
