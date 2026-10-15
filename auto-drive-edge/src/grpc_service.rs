use std::{convert::Infallible, pin::Pin, sync::Arc};

use futures_util::{Stream, StreamExt, stream};
use the_beginner_car_tcp_protocol::TheBeginnerCarOutgoingMessages;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status};

use crate::{
    auto_drive_edge_service::v1::{
        AutoScriptRunningEvent, AutoScriptStoppedEvent, BatteryReadingEvent,
        CancelAutoScriptRequest, CancelAutoScriptResponse, InfraredLedReadingsEvent, LogEvent,
        LogLevel, LogMessage, LogSystemOrigin, MotorSpeedUpdatedEvent, RunAutoScriptRequest,
        RunAutoScriptResponse, SystemStateEvent, WasmInstalledEvent,
        WatchTheBeginnerCarEventsRequest, WatchTheBeginnerCarEventsResponse,
        auto_drive_edge_service_server::AutoDriveEdgeService, run_auto_script_response,
        watch_the_beginner_car_events_response::Event,
    },
    the_beginner_car_auto_script::TheBeginnerCarAutoScript,
};

pub struct GrpcService {
    build_a: Arc<TheBeginnerCarAutoScript>,
}

impl GrpcService {
    pub fn new(build_a: Arc<TheBeginnerCarAutoScript>) -> Self {
        Self { build_a }
    }
}

#[tonic::async_trait]
impl AutoDriveEdgeService for GrpcService {
    type RunAutoScriptStream =
        Pin<Box<dyn Stream<Item = Result<RunAutoScriptResponse, Status>> + Send + 'static>>;

    async fn run_auto_script(
        &self,
        request: Request<RunAutoScriptRequest>,
    ) -> Result<Response<Self::RunAutoScriptStream>, Status> {
        let run_uuid = self
            .build_a
            .enqueue_auto_script(request.into_inner().data)
            .await;

        let receiver = self.build_a.incoming_rx.subscribe();

        let event_stream = stream::once(async {
            Ok(RunAutoScriptResponse {
                event: Some(run_auto_script_response::Event::RunUuid(run_uuid)),
            })
        });

        let stream = event_stream.chain(BroadcastStream::new(receiver).filter_map(
            |result| async move {
                match result {
                    Ok(message) => match message {
                        TheBeginnerCarOutgoingMessages::Log { log } => {
                            let log_level = match log.level {
                                the_beginner_car_tcp_protocol::LogLevel::Error => LogLevel::Error,
                                the_beginner_car_tcp_protocol::LogLevel::Warn => LogLevel::Warn,
                                the_beginner_car_tcp_protocol::LogLevel::Info => LogLevel::Info,
                                the_beginner_car_tcp_protocol::LogLevel::Debug => LogLevel::Debug,
                                the_beginner_car_tcp_protocol::LogLevel::Trace => LogLevel::Trace,
                            };

                            let system_origin = match log.system_origin {
                                the_beginner_car_tcp_protocol::LogSystemOrigin::WasmMcu => {
                                    LogSystemOrigin::WasmMcu
                                }
                                the_beginner_car_tcp_protocol::LogSystemOrigin::WifiMcu => {
                                    LogSystemOrigin::WifiMcu
                                }
                            };

                            let log = LogMessage {
                                system_origin: system_origin as i32,
                                level: log_level as i32,
                                message: log.message,
                            };
                            Some(Ok(RunAutoScriptResponse {
                                event: Some(run_auto_script_response::Event::Log(LogEvent {
                                    log: Some(log),
                                })),
                            }))
                        }
                        TheBeginnerCarOutgoingMessages::SystemState {} => None,
                        TheBeginnerCarOutgoingMessages::MotorASpeedUpdated { speed } => None,
                        TheBeginnerCarOutgoingMessages::MotorBSpeedUpdated { speed } => None,
                        TheBeginnerCarOutgoingMessages::WasmInstalled => None,
                        TheBeginnerCarOutgoingMessages::AutoScriptRunning => None,
                        TheBeginnerCarOutgoingMessages::AutoScriptStopped => None,
                        TheBeginnerCarOutgoingMessages::BatteryReading {
                            motor_battery_voltage,
                            mcu_battery_voltage,
                        } => None,
                        TheBeginnerCarOutgoingMessages::InfraredLedReadings {
                            utmost_front_utmost_left,
                            utmost_front_utmost_right,
                        } => None,
                    },
                    Err(_) => {
                        // BroadcastStream reports lagged receivers here.
                        // Skip lost messages and continue streaming.
                        None
                    }
                }
            },
        ));

        Ok(Response::new(Box::pin(stream)))
    }

    async fn cancel_auto_script(
        &self,
        request: Request<CancelAutoScriptRequest>,
    ) -> Result<Response<CancelAutoScriptResponse>, Status> {
        self.build_a
            .cancel_auto_script(request.into_inner().run_uuid)
            .await;

        Ok(Response::new(CancelAutoScriptResponse {}))
    }

    type WatchTheBeginnerCarEventsStream = Pin<
        Box<dyn Stream<Item = Result<WatchTheBeginnerCarEventsResponse, Status>> + Send + 'static>,
    >;

    async fn watch_the_beginner_car_events(
        &self,
        _request: Request<WatchTheBeginnerCarEventsRequest>,
    ) -> Result<Response<Self::WatchTheBeginnerCarEventsStream>, Status> {
        // Replace this with the driver's independent event subscription.

        let receiver = self.build_a.incoming_rx.subscribe();

        let stream = BroadcastStream::new(receiver).filter_map(|result| async move {
            match result {
                Ok(message) => {
                    // Convert TheBeginnerCarOutgoingMessages to the
                    // generated protobuf oneof response here.
                    Some(Ok(convert_message(message)))
                }
                Err(_) => {
                    // BroadcastStream reports lagged receivers here.
                    // Skip lost messages and continue streaming.
                    None
                }
            }
        });

        Ok(Response::new(Box::pin(stream)))
    }
}

fn convert_message(message: TheBeginnerCarOutgoingMessages) -> WatchTheBeginnerCarEventsResponse {
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
