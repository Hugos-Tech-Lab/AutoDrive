use std::pin::Pin;

use futures_util::{Stream, StreamExt};
use the_beginner_car_tcp_protocol::{TheBeginnerCarOutgoingMessages};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status};

use crate::{
    auto_drive_edge_service::v1::{
        LogEvent, LogMessage, RunAutoScriptRequest, RunAutoScriptResponse, WatchTheBeginnerCarEventsRequest, WatchTheBeginnerCarEventsResponse, auto_drive_edge_service_server::AutoDriveEdgeService, watch_the_beginner_car_events_response::Event,
    }, the_beginner_car_driver::TheBeginnerCarDriver,
};

pub struct GrpcService {
    build_a: TheBeginnerCarDriver,
}

impl GrpcService {
    pub fn new(build_a: TheBeginnerCarDriver) -> Self {
        Self { build_a }
    }
}

#[tonic::async_trait]
impl AutoDriveEdgeService for GrpcService {
    async fn run_auto_script(
        &self,
        request: Request<RunAutoScriptRequest>,
    ) -> Result<Response<RunAutoScriptResponse>, Status> {
        // let request = request.into_inner();

        // Adapt the payload to your actual script format.
        // let _script = request.name.into_bytes();

        unimplemented!();
        // Ok(Response::new(RunAutoScriptResponse {}))
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
        TheBeginnerCarOutgoingMessages::Log { log } => WatchTheBeginnerCarEventsResponse {
            event: Some(Event::Log(LogEvent {
                log: Some(LogMessage {
                    message: log.message
                }),
            })),
        },
        TheBeginnerCarOutgoingMessages::SystemState {} => todo!(),
        TheBeginnerCarOutgoingMessages::MotorASpeedUpdated { speed } => todo!(),
        TheBeginnerCarOutgoingMessages::MotorBSpeedUpdated { speed } => todo!(),
        TheBeginnerCarOutgoingMessages::WasmInstalled => todo!(),
        TheBeginnerCarOutgoingMessages::AutoScriptRunning => todo!(),
        TheBeginnerCarOutgoingMessages::AutoScriptStopped => todo!(),
        TheBeginnerCarOutgoingMessages::BatteryReading {
            motor_battery_voltage,
            mcu_battery_voltage,
        } => todo!(),
        TheBeginnerCarOutgoingMessages::IRSensorAReading { value } => todo!(),
    }
}
