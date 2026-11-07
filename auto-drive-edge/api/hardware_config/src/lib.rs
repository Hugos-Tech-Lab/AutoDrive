pub mod config;

#[derive(Debug, Clone)]
pub struct WifiMcuMetadata {
    pub ip: &'static str,
    pub mac_address: &'static str,
}

#[derive(Debug, Clone)]
pub struct WasmMcuMetadata {
    pub mac_address: &'static str,
}

#[derive(Debug, Clone)]
pub struct TheBeginnerCarMetadata {
    pub number_plate: &'static str,
    pub aruco_marker_id: u32,
    pub wifi_mcu: WifiMcuMetadata,
    pub wasm_mcu: WasmMcuMetadata,
}

#[derive(Debug)]
pub struct HardwareConfig {
    pub the_beginner_cars: &'static [TheBeginnerCarMetadata],
}
