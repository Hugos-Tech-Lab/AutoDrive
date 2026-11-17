use esp_idf_sys::xTaskGetTickCount;

pub fn uptime_in_ms() -> u32 {
    const _: () = assert!(
        esp_idf_svc::sys::CONFIG_FREERTOS_HZ == 1000,
        "CONFIG_FREERTOS_HZ must be set to 1000 in sdkconfig.defaults OR you should double check the logic below, the tick count may not be in milliseconds anymore"
    );
    unsafe { xTaskGetTickCount() }
}
