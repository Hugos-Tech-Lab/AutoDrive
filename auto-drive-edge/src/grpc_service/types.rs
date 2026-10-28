use the_beginner_car_tcp_protocol::TheBeginnerCarOutgoingMessages;

// Ensure you have these imported from your proto module
use crate::auto_drive_edge_service::{
    self,
    v1::{
        AutoScriptRunningEvent, AutoScriptStoppedEvent, BatteryReadingEvent,
        InfraredLedReadingsEvent, LogEvent, LogLevel, LogSystemOrigin, MotorSpeedUpdatedEvent,
        SystemStateEvent, WasmInstalledEvent, WatchTheBeginnerCarEventsResponse,
        watch_the_beginner_car_events_response::Event,
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
    //
}

impl From<TheBeginnerCarOutgoingMessages> for WatchTheBeginnerCarEventsResponse {
    fn from(message: TheBeginnerCarOutgoingMessages) -> Self {
        match message {
            TheBeginnerCarOutgoingMessages::Log { log } => WatchTheBeginnerCarEventsResponse {
                event: Some(Event::Log(LogEvent {
                    log: Some(to_grpc_log_message(log)),
                })),
            },
            TheBeginnerCarOutgoingMessages::SystemState {} => WatchTheBeginnerCarEventsResponse {
                event: Some(Event::SystemState(SystemStateEvent {})),
            },
            TheBeginnerCarOutgoingMessages::MotorASpeedUpdated { speed } => {
                WatchTheBeginnerCarEventsResponse {
                    event: Some(Event::MotorASpeedUpdated(MotorSpeedUpdatedEvent {
                        speed: speed as i32,
                    })),
                }
            }
            TheBeginnerCarOutgoingMessages::MotorBSpeedUpdated { speed } => {
                WatchTheBeginnerCarEventsResponse {
                    event: Some(Event::MotorBSpeedUpdated(MotorSpeedUpdatedEvent {
                        speed: speed as i32,
                    })),
                }
            }

            TheBeginnerCarOutgoingMessages::WasmInstalled => WatchTheBeginnerCarEventsResponse {
                event: Some(Event::WasmInstalled(WasmInstalledEvent {})),
            },

            TheBeginnerCarOutgoingMessages::AutoScriptRunning => {
                WatchTheBeginnerCarEventsResponse {
                    event: Some(Event::AutoScriptRunning(AutoScriptRunningEvent {})),
                }
            }

            TheBeginnerCarOutgoingMessages::AutoScriptStopped => {
                WatchTheBeginnerCarEventsResponse {
                    event: Some(Event::AutoScriptStopped(AutoScriptStoppedEvent {})),
                }
            }

            TheBeginnerCarOutgoingMessages::BatteryReading { voltage } => {
                WatchTheBeginnerCarEventsResponse {
                    event: Some(Event::BatteryReading(BatteryReadingEvent {
                        battery_voltage: voltage,
                    })),
                }
            }

            TheBeginnerCarOutgoingMessages::InfraredLedReadings {
                back_center,
                front_left,
                front_center,
                front_right,
            } => WatchTheBeginnerCarEventsResponse {
                event: Some(Event::InfraredLedReading(InfraredLedReadingsEvent {
                    utmost_front_utmost_left: back_center as u32,
                    utmost_front_utmost_right: front_left as u32,
                })),
            },
        }
    }
}

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
            number_plate: value.hardware.number_plate.clone(),
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
