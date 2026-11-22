use the_beginner_car_tcp_protocol::TheBeginnerCarOutgoingMessages;

// Ensure you have these imported from your proto module
use crate::auto_drive_edge_service::{
    self,
    v1::{
        AutoScriptRunningEvent, AutoScriptStoppedEvent, BatteryReadingEvent,
        InfraredLedReadingsEvent, LogLevel, LogSystemOrigin, MotorSpeedUpdatedEvent,
        SystemStateEvent, WasmInstalledEvent,
    },
};

pub fn to_grpc_log_message(
    log_message: auto_drive_common_types::LogMessage,
) -> auto_drive_edge_service::v1::LogMessage {
    let log_level = match log_message.level {
        auto_drive_common_types::LogLevel::Error => LogLevel::Error,
        auto_drive_common_types::LogLevel::Warn => LogLevel::Warn,
        auto_drive_common_types::LogLevel::Info => LogLevel::Info,
        auto_drive_common_types::LogLevel::Debug => LogLevel::Debug,
        auto_drive_common_types::LogLevel::Trace => LogLevel::Trace,
    };

    let system_origin = match log_message.system_origin {
        auto_drive_common_types::LogSystemOrigin::WasmMcu => LogSystemOrigin::WasmMcu,
        auto_drive_common_types::LogSystemOrigin::WifiMcu => LogSystemOrigin::WifiMcu,
        auto_drive_common_types::LogSystemOrigin::CloudServer => LogSystemOrigin::WifiMcu,
        auto_drive_common_types::LogSystemOrigin::EdgeServer => LogSystemOrigin::WifiMcu,
    };

    crate::auto_drive_edge_service::v1::LogMessage {
        message: log_message.message,
        level: log_level as i32,
        system_origin: system_origin as i32,
    }
}

// pub fn from_grpc_log_message(
//     log_message: auto_drive_edge_service::v1::LogMessage,
// ) -> auto_drive_common_types::LogMessage {
//     let log_level = match log_message.level {
//         auto_drive_common_types::LogLevel::Error => LogLevel::Error,
//         auto_drive_common_types::LogLevel::Warn => LogLevel::Warn,
//         auto_drive_common_types::LogLevel::Info => LogLevel::Info,
//         auto_drive_common_types::LogLevel::Debug => LogLevel::Debug,
//         auto_drive_common_types::LogLevel::Trace => LogLevel::Trace,
//     };

//     let system_origin = match log_message.system_origin {
//         auto_drive_common_types::LogSystemOrigin::WasmMcu => LogSystemOrigin::WasmMcu,
//         auto_drive_common_types::LogSystemOrigin::WifiMcu => LogSystemOrigin::WifiMcu,
//         auto_drive_common_types::LogSystemOrigin::CloudServer => LogSystemOrigin::WifiMcu,
//         auto_drive_common_types::LogSystemOrigin::EdgeServer => LogSystemOrigin::WifiMcu,
//     };

//     crate::auto_drive_edge_service::v1::LogMessage {
//         message: log_message.message,
//         level: log_level as i32,
//         system_origin: system_origin as i32,
//     }
// }

impl From<&crate::auto_script_manager::auto_script::AutoScript>
    for crate::auto_drive_edge_service::v1::AutoScript
{
    fn from(value: &crate::auto_script_manager::auto_script::AutoScript) -> Self {
        Self {
            owner_name: value.owner_name.clone(),
            owner_comment: value.owner_comment.clone(),
            uploaded_script_hash: value.uploaded_script_hash.clone(),
            script_queue_time: value.script_queue_time.to_string(),
            script_start_time: value.script_start_time.map(|v| v.to_string()),
            vehicle_start_time: value.vehicle_start_time.map(|v| v.to_string()),
            vehicle_finish_time: value.vehicle_finish_time.map(|v| v.to_string()),
            script_stop_time: value.script_stop_time.map(|v| v.to_string()),
            run_number: value.run_number,
            run_uuid: value.run_uuid.clone(),
            number_plate: value
                .hardware
                .as_ref()
                .map(|comms| comms.metadata.number_plate.to_string().clone()),
            script_type: match value.script_type {
                crate::auto_script_manager::auto_script::AutoScriptType::TimeTrial => {
                    auto_drive_edge_service::v1::AutoScriptType::TimeTrials as i32
                }
                crate::auto_script_manager::auto_script::AutoScriptType::Reposition => {
                    auto_drive_edge_service::v1::AutoScriptType::Reposition as i32
                }
                crate::auto_script_manager::auto_script::AutoScriptType::ConfigAdjust => {
                    auto_drive_edge_service::v1::AutoScriptType::ConfigAdjust as i32
                }
            },
        }
    }
}

impl From<crate::auto_script_manager::AutoScripts>
    for crate::auto_drive_edge_service::v1::AutoScripts
{
    fn from(value: crate::auto_script_manager::AutoScripts) -> Self {
        Self {
            running: value.running.iter().map(|v| v.into()).collect(),
            queued: value.queued.iter().map(|v| v.into()).collect(),
            stopped: value.stopped.iter().map(|v| v.into()).collect(),
        }
    }
}
