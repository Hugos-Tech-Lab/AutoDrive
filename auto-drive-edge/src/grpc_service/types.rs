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
    log_message: the_beginner_car_tcp_protocol::LogMessage,
) -> auto_drive_edge_service::v1::LogMessage {
    let log_level = match log_message.level {
        the_beginner_car_tcp_protocol::LogLevel::Error => LogLevel::Error,
        the_beginner_car_tcp_protocol::LogLevel::Warn => LogLevel::Warn,
        the_beginner_car_tcp_protocol::LogLevel::Info => LogLevel::Info,
        the_beginner_car_tcp_protocol::LogLevel::Debug => LogLevel::Debug,
        the_beginner_car_tcp_protocol::LogLevel::Trace => LogLevel::Trace,
    };

    let system_origin = match log_message.system_origin {
        the_beginner_car_tcp_protocol::LogSystemOrigin::WasmMcu => LogSystemOrigin::WasmMcu,
        the_beginner_car_tcp_protocol::LogSystemOrigin::WifiMcu => LogSystemOrigin::WifiMcu,
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

            TheBeginnerCarOutgoingMessages::BatteryReading {
                motor_battery_voltage,
                mcu_battery_voltage,
            } => WatchTheBeginnerCarEventsResponse {
                event: Some(Event::BatteryReading(BatteryReadingEvent {
                    motor_battery_voltage,
                    mcu_battery_voltage,
                })),
            },

            TheBeginnerCarOutgoingMessages::InfraredLedReadings {
                utmost_front_utmost_left,
                utmost_front_utmost_right,
            } => WatchTheBeginnerCarEventsResponse {
                event: Some(Event::InfraredLedReading(InfraredLedReadingsEvent {
                    utmost_front_utmost_left: utmost_front_utmost_left as u32,
                    utmost_front_utmost_right: utmost_front_utmost_right as u32,
                })),
            },
        }
    }
}

impl From<&crate::auto_script_manager::AutoScript>
    for crate::auto_drive_edge_service::v1::AutoScript
{
    fn from(value: &crate::auto_script_manager::AutoScript) -> Self {
        Self {
            owner_name: value.owner_name.clone(),
        }
    }
}

impl From<crate::auto_script_manager::AutoScriptQueueSnapshot>
    for crate::auto_drive_edge_service::v1::AllAutoScripts
{
    fn from(value: crate::auto_script_manager::AutoScriptQueueSnapshot) -> Self {
        Self {
            running_scripts: value.running_scripts.iter().map(|v| v.into()).collect(),
            queued_scripts: value.queued_scripts.iter().map(|v| v.into()).collect(),
            stopped_scripts: value.stopped_scripts.iter().map(|v| v.into()).collect(),
        }
    }
}
