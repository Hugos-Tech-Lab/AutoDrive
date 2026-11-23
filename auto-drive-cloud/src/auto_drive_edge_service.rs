use auto_drive_edge_protocol::auto_drive_edge_service::v1::{self, LogMessage};

pub fn format_log(log_message: &LogMessage) -> String {
    let level = match log_message.level() {
        v1::LogLevel::Unspecified => "UNSPECIFIED",
        v1::LogLevel::Error => "ERROR",
        v1::LogLevel::Warn => "WARN",
        v1::LogLevel::Info => "INFO",
        v1::LogLevel::Debug => "DEBUG",
        v1::LogLevel::Trace => "TRACE",
    };

    let system_origin = match log_message.system_origin() {
        v1::LogSystemOrigin::WasmUnspecified => "UNSPECIFIED",
        v1::LogSystemOrigin::WasmMcu => "WASM_MCU",
        v1::LogSystemOrigin::WifiMcu => "WIFI_MCU",
    };

    format!("[{}] [{}] {}", level, system_origin, log_message.message)
}
