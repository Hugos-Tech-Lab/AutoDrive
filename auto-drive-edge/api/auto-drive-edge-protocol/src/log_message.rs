use crate::auto_drive_edge_service::v1::{self, LogLevel, LogSystemOrigin};

impl From<auto_drive_common_types::LogMessage> for v1::LogMessage {
    fn from(value: auto_drive_common_types::LogMessage) -> Self {
        Self {
            system_origin: LogSystemOrigin::from(value.system_origin).into(),
            level: LogLevel::from(value.level).into(),
            message: value.message,
        }
    }
}

impl TryFrom<v1::LogMessage> for auto_drive_common_types::LogMessage {
    type Error = ();

    fn try_from(value: v1::LogMessage) -> Result<Self, Self::Error> {
        let system_origin = v1::LogSystemOrigin::try_from(value.system_origin).map_err(|_| ())?;
        let system_origin = auto_drive_common_types::LogSystemOrigin::try_from(system_origin)?;

        let level = v1::LogLevel::try_from(value.level).map_err(|_| ())?;
        let level = auto_drive_common_types::LogLevel::try_from(level)?;

        Ok(Self {
            system_origin,
            level,
            target: "".to_string(),
            message: value.message,
            run_uuid: None,
        })
    }
    //
}

// pub fn to_grpc_log_message(
//     log_message: auto_drive_common_types::LogMessage,
// ) -> auto_drive_edge_service::v1::LogMessage {
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
//         auto_drive_common_types::LogSystemOrigin::EdgeServer => LogSystemOrigin::WifiMcu,
//     };

//     crate::auto_drive_edge_service::v1::LogMessage {
//         message: log_message.message,
//         level: log_level as i32,
//         system_origin: system_origin as i32,
//     }
// }

// // pub fn from_grpc_log_message(
// //     log_message: auto_drive_edge_service::v1::LogMessage,
// // ) -> auto_drive_common_types::LogMessage {
// //     let log_level = match log_message.level {
// //         auto_drive_common_types::LogLevel::Error => LogLevel::Error,
// //         auto_drive_common_types::LogLevel::Warn => LogLevel::Warn,
// //         auto_drive_common_types::LogLevel::Info => LogLevel::Info,
// //         auto_drive_common_types::LogLevel::Debug => LogLevel::Debug,
// //         auto_drive_common_types::LogLevel::Trace => LogLevel::Trace,
// //     };

// //     let system_origin = match log_message.system_origin {
// //         auto_drive_common_types::LogSystemOrigin::WasmMcu => LogSystemOrigin::WasmMcu,
// //         auto_drive_common_types::LogSystemOrigin::WifiMcu => LogSystemOrigin::WifiMcu,
// //         auto_drive_common_types::LogSystemOrigin::CloudServer => LogSystemOrigin::WifiMcu,
// //         auto_drive_common_types::LogSystemOrigin::EdgeServer => LogSystemOrigin::WifiMcu,
// //     };

// //     crate::auto_drive_edge_service::v1::LogMessage {
// //         message: log_message.message,
// //         level: log_level as i32,
// //         system_origin: system_origin as i32,
// //     }
// // }
