use the_beginner_car_tcp_protocol::TheBeginnerCarOutgoingMessages;

use crate::auto_drive_edge_service::{
    self,
    v1::{LogLevel, LogSystemOrigin},
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

impl From<&crate::auto_script::AutoScript> for crate::auto_drive_edge_service::v1::AutoScript {
    fn from(value: &crate::auto_script::AutoScript) -> Self {
        Self {
            owner_name: value.owner_name.clone(),
            owner_comment: value.owner_comment.clone(),
            uploaded_script_hash: value.uploaded_script_hash.clone(),
            script_queue_time: value.script_queue_time.to_string(),
            script_start_time: value.script_start_time.map(|v| v.to_string()),
            vehicle_start_time: value.time_trials_start_time.map(|v| v.to_string()),
            vehicle_finish_time: value.time_trials_finish_time.map(|v| v.to_string()),
            script_stop_time: value.script_stop_time.map(|v| v.to_string()),
            run_number: value.run_number,
            run_uuid: value.run_uuid.clone(),
            number_plate: value.vehicle_number_plate.clone(),
            script_type: 0,
            lifecycle: match value.lifecycle {
                crate::auto_scripts::AutoScriptLifecycle::Idle => 0,
                crate::auto_scripts::AutoScriptLifecycle::SendingToWifiMcu => 0,
                crate::auto_scripts::AutoScriptLifecycle::ReceivedByWifiMcu => 0,
                crate::auto_scripts::AutoScriptLifecycle::SendingToWasmMcu => 0,
                crate::auto_scripts::AutoScriptLifecycle::ReceivedByWasmMcu => 0,
                crate::auto_scripts::AutoScriptLifecycle::Cancelling => 0,
                crate::auto_scripts::AutoScriptLifecycle::StartedCallingMain => 0,
                crate::auto_scripts::AutoScriptLifecycle::StoppedByScriptEnd => 0,
                crate::auto_scripts::AutoScriptLifecycle::StoppedBySafety => 0,
                crate::auto_scripts::AutoScriptLifecycle::StoppedByHttpCancel => 0,
                crate::auto_scripts::AutoScriptLifecycle::StoppedByDisconnect => 0,
            },
            runtime_lifecycle: 0,
        }
    }
}
