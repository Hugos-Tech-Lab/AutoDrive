use crate::auto_drive_edge_service::v1::LogMessage;

pub mod v1 {
    tonic::include_proto!("auto_drive_edge_service.v1");
}

pub fn format_log(log_message: &LogMessage) -> String {
    let level = match log_message.level() {
        crate::auto_drive_edge_service::v1::LogLevel::Unspecified => "UNSPECIFIED",
        crate::auto_drive_edge_service::v1::LogLevel::Error => "ERROR",
        crate::auto_drive_edge_service::v1::LogLevel::Warn => "WARN",
        crate::auto_drive_edge_service::v1::LogLevel::Info => "INFO",
        crate::auto_drive_edge_service::v1::LogLevel::Debug => "DEBUG",
        crate::auto_drive_edge_service::v1::LogLevel::Trace => "TRACE",
    };

    let system_origin = match log_message.system_origin() {
        crate::auto_drive_edge_service::v1::LogSystemOrigin::WasmUnspecified => "UNSPECIFIED",
        crate::auto_drive_edge_service::v1::LogSystemOrigin::WasmMcu => "WASM_MCU",
        crate::auto_drive_edge_service::v1::LogSystemOrigin::WifiMcu => "WIFI_MCU",
    };

    format!("[{}] [{}] {}", level, system_origin, log_message.message)
}
