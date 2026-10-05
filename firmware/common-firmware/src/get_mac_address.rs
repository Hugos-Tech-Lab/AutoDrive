use esp_idf_sys::{esp, esp_efuse_mac_get_default};

pub fn get_mac_address() -> Result<[u8; 6], esp_idf_sys::EspError> {
    let mut mac = [0u8; 6];

    unsafe {
        esp!(esp_efuse_mac_get_default(mac.as_mut_ptr()))?;
    }

    Ok(mac)
}

pub fn format_mac_address(mac: &[u8; 6]) -> String {
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}
