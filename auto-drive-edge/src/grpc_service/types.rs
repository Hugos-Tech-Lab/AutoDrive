use the_beginner_car_tcp_protocol::TheBeginnerCarOutgoingMessages;

// Ensure you have these imported from your proto module
use crate::auto_drive_edge_service::v1::{
    AutoScriptRunningEvent, AutoScriptStoppedEvent, BatteryReadingEvent, 
    InfraredLedReadingsEvent, LogEvent, LogLevel, LogMessage, LogSystemOrigin, 
    MotorSpeedUpdatedEvent, SystemStateEvent, WasmInstalledEvent, 
    WatchTheBeginnerCarEventsResponse, watch_the_beginner_car_events_response::Event,
};

impl From<TheBeginnerCarOutgoingMessages> for WatchTheBeginnerCarEventsResponse {
    fn from(message: TheBeginnerCarOutgoingMessages) -> Self {
        match message {
            TheBeginnerCarOutgoingMessages::Log { log } => {
                let log_level = match log.level {
                    the_beginner_car_tcp_protocol::LogLevel::Error => LogLevel::Error,
                    the_beginner_car_tcp_protocol::LogLevel::Warn => LogLevel::Warn,
                    the_beginner_car_tcp_protocol::LogLevel::Info => LogLevel::Info,
                    the_beginner_car_tcp_protocol::LogLevel::Debug => LogLevel::Debug,
                    the_beginner_car_tcp_protocol::LogLevel::Trace => LogLevel::Trace,
                };

                let system_origin = match log.system_origin {
                    the_beginner_car_tcp_protocol::LogSystemOrigin::WasmMcu => LogSystemOrigin::WasmMcu,
                    the_beginner_car_tcp_protocol::LogSystemOrigin::WifiMcu => LogSystemOrigin::WifiMcu,
                };

                WatchTheBeginnerCarEventsResponse {
                    event: Some(Event::Log(LogEvent {
                        log: Some(LogMessage {
                            message: log.message,
                            level: log_level as i32,
                            system_origin: system_origin as i32,
                        }),
                    })),
                }
            }

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

            TheBeginnerCarOutgoingMessages::AutoScriptRunning => WatchTheBeginnerCarEventsResponse {
                event: Some(Event::AutoScriptRunning(AutoScriptRunningEvent {})),
            },

            TheBeginnerCarOutgoingMessages::AutoScriptStopped => WatchTheBeginnerCarEventsResponse {
                event: Some(Event::AutoScriptStopped(AutoScriptStoppedEvent {})),
            },

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
