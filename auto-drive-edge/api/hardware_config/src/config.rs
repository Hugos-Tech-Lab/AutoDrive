use crate::{HardwareConfig, TheBeginnerCarMetadata, WasmMcuMetadata, WifiMcuMetadata};

pub const HARDWARE_CONFIG: HardwareConfig = HardwareConfig {
    the_beginner_cars: &[TheBeginnerCarMetadata {
        number_plate: "TBC 001",
        aruco_marker_id: 0,
        wifi_mcu: WifiMcuMetadata {
            ip: "192.168.0.197",
            mac_address: "9C-CC-01-43-E0-94",
        },
        wasm_mcu: WasmMcuMetadata {
            mac_address: "00-00-00-00-00",
        },
    }],
};
