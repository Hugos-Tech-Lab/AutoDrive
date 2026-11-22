use std::{pin::Pin, sync::Arc};

use futures_util::{Stream, StreamExt};
use the_beginner_car_tcp_protocol::TheBeginnerCarOutgoingMessages::Position;
use tokio::sync::mpsc;
use tokio_stream::wrappers::{BroadcastStream, ReceiverStream, WatchStream};
use tonic::{Request, Response, Status};

use crate::{
    auto_drive_edge_service::v1::{
        CancelAutoScriptRequest, CancelAutoScriptResponse, EnqueueAutoScriptRequest,
        EnqueueAutoScriptResponse, GetVehiclesRequest, GetVehiclesResponse, Hardware,
        NotifyHardwareCoordinatesRequest, NotifyHardwareCoordinatesResponse, RunAutoScriptRequest,
        RunAutoScriptResponse, WatchAutoScriptLogsRequest, WatchAutoScriptLogsResponse,
        WatchAutoScriptOverviewRequest, WatchAutoScriptOverviewResponse, WatchHardwarePoseRequest,
        WatchHardwarePoseResponse, WatchHardwareStatusRequest, WatchHardwareStatusResponse,
        WatchVehicleLogsRequest, WatchVehicleLogsResponse,
        auto_drive_edge_service_server::AutoDriveEdgeService,
    },
    auto_script_manager::{AutoScriptManager, auto_script::AutoScriptType},
    grpc_service::types::to_grpc_log_message,
    the_beginner_car::TheBeginnerCar,
};

pub mod types;

pub struct GrpcService {
    auto_script_manager: Arc<AutoScriptManager>,
    the_beginner_cars: Vec<TheBeginnerCar>,
}

impl GrpcService {
    pub fn new(
        auto_script_manager: Arc<AutoScriptManager>,
        the_beginner_cars: Vec<TheBeginnerCar>,
    ) -> Self {
        Self {
            auto_script_manager,
            the_beginner_cars,
        }
    }
}

#[tonic::async_trait]
impl AutoDriveEdgeService for GrpcService {
    async fn enqueue_auto_script(
        &self,
        request: Request<EnqueueAutoScriptRequest>,
    ) -> Result<Response<EnqueueAutoScriptResponse>, Status> {
        let request = request.into_inner();
        let run_uuid = self.auto_script_manager.enqueue_auto_script(
            request.owner_name,
            request.owner_comment,
            request.data,
            AutoScriptType::TimeTrial,
        );

        Ok(Response::new(EnqueueAutoScriptResponse { run_uuid }))
    }

    async fn get_vehicles(
        &self,
        _request: Request<GetVehiclesRequest>,
    ) -> Result<Response<GetVehiclesResponse>, Status> {
        let number_plates = self
            .the_beginner_cars
            .iter()
            .map(|vehicle| vehicle.metadata.number_plate.to_string())
            .collect();
        Ok(Response::new(GetVehiclesResponse { number_plates }))
    }

    async fn run_auto_script(
        &self,
        request: Request<RunAutoScriptRequest>,
    ) -> Result<Response<RunAutoScriptResponse>, Status> {
        let request = request.into_inner();
        let run_uuid = self
            .auto_script_manager
            .run_auto_script(request.run_uuid, request.number_plate)
            .await;
        // TODO: handle result
        Ok(Response::new(RunAutoScriptResponse {}))
    }

    async fn cancel_auto_script(
        &self,
        request: Request<CancelAutoScriptRequest>,
    ) -> Result<Response<CancelAutoScriptResponse>, Status> {
        self.auto_script_manager
            .cancel_auto_script(request.into_inner().run_uuid)
            .await;

        Ok(Response::new(CancelAutoScriptResponse {}))
    }

    type WatchAutoScriptLogsStream =
        Pin<Box<dyn Stream<Item = Result<WatchAutoScriptLogsResponse, Status>> + Send + 'static>>;

    async fn watch_auto_script_logs(
        &self,
        request: Request<WatchAutoScriptLogsRequest>,
    ) -> Result<Response<Self::WatchAutoScriptLogsStream>, Status> {
        let run_uuid = request.into_inner().run_uuid;

        let (past_events, live_rx) = self
            .auto_script_manager
            .watch_auto_script(run_uuid)
            .await
            .ok_or_else(|| Status::not_found("Script run not found or already stopped"))?;

        let history_stream = tokio_stream::iter(past_events).map(|event| {
            let res;
            match event {
                crate::auto_script_manager::AutoScriptEvents::LogMessage { log_message } => res = WatchAutoScriptResponse {
                event: Some(crate::auto_drive_edge_service::v1::watch_auto_script_response::Event::LogMessage(to_grpc_log_message(log_message.clone())))
                },
                crate::auto_script_manager::AutoScriptEvents::ScriptExited => res = WatchAutoScriptResponse {
                event: Some(crate::auto_drive_edge_service::v1::watch_auto_script_response::Event::ScriptExited(0))
                },
            };
            Ok(res)
        });

        let live_stream = BroadcastStream::new(live_rx).filter_map(|result| async move {
            match result {
                Ok(event) => {
                    let res;
                    match event {
                            crate::auto_script_manager::AutoScriptEvents::LogMessage { log_message } => res = WatchAutoScriptResponse {
                            event: Some(crate::auto_drive_edge_service::v1::watch_auto_script_response::Event::LogMessage(to_grpc_log_message(log_message.clone())))
                        },
                        crate::auto_script_manager::AutoScriptEvents::ScriptExited => res = WatchAutoScriptResponse {
                        event: Some(crate::auto_drive_edge_service::v1::watch_auto_script_response::Event::ScriptExited(0))
                        },
                    };
                    Some(Ok(res))
                },
                Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(skipped)) => {
                    eprintln!("Watch stream lagged; skipped {skipped} messages");
                    None
                }
            }
        });

        let combined_stream = history_stream.chain(live_stream);

        Ok(Response::new(Box::pin(combined_stream)))
    }

    type WatchAutoScriptOverviewStream = Pin<
        Box<dyn Stream<Item = Result<WatchAutoScriptOverviewResponse, Status>> + Send + 'static>,
    >;

    async fn watch_auto_script_overview(
        &self,
        _request: Request<WatchAutoScriptOverviewRequest>,
    ) -> Result<Response<Self::WatchAutoScriptOverviewStream>, Status> {
        let (_snapshot, queue_receiver) = self.auto_script_manager.watch_auto_script_overview();

        let scripts = WatchStream::new(queue_receiver).map(|snapshot| {
            Ok(WatchAutoScriptOverviewResponse {
                scripts: Some(snapshot.into()),
            })
        });

        Ok(Response::new(Box::pin(scripts)))
    }

    type WatchVehicleLogsStream =
        Pin<Box<dyn Stream<Item = Result<WatchVehicleLogsResponse, Status>> + Send + 'static>>;

    async fn watch_vehicle_logs(
        &self,
        request: Request<WatchVehicleLogsRequest>,
    ) -> Result<Response<Self::WatchVehicleLogsStream>, Status> {
        // TODO: find the ID from the request
        // let vehicle = self.the_beginner_cars.first().unwrap();

        // let stream =
        //     BroadcastStream::new(vehicle.incoming_tx.subscribe()).filter_map(|result| async move {
        //         match result {
        //             Ok(message) => Some(Ok(message.into())),
        //             Err(_) => {
        //                 // BroadcastStream reports lagged receivers here.
        //                 // Skip lost messages and continue streaming.
        //                 None
        //             }
        //         }
        //     });

        // Ok(Response::new(Box::pin(stream)))

        todo!()
    }

    type WatchHardwareStatusStream =
        Pin<Box<dyn Stream<Item = Result<WatchHardwareStatusResponse, Status>> + Send + 'static>>;

    async fn watch_hardware_status(
        &self,
        _request: Request<WatchHardwareStatusRequest>,
    ) -> Result<Response<Self::WatchHardwareStatusStream>, Status> {
        let (tx, rx) = mpsc::channel(128);

        let mut initial_hardware = Vec::new();

        for car_entry in self.the_beginner_cars.iter() {
            let comms = car_entry;
            initial_hardware.push(Hardware {
                number_plate: comms.metadata.number_plate.to_string(),
                category: "TheBeginnerCar".to_string(),
                is_connected: *comms.connected_rx.borrow(),
            });
        }

        let initial_msg = WatchHardwareStatusResponse {
            hardware: initial_hardware,
        };

        if tx.send(Ok(initial_msg)).await.is_err() {
            return Ok(Response::new(Box::pin(ReceiverStream::new(rx))));
        }

        for car_entry in self.the_beginner_cars.iter() {
            let comms = car_entry.clone();
            let mut is_connected_rx = comms.connected_rx.clone();
            let tx_clone = tx.clone();

            tokio::spawn(async move {
                while is_connected_rx.changed().await.is_ok() {
                    let current_state = *is_connected_rx.borrow();

                    let update_msg = WatchHardwareStatusResponse {
                        hardware: vec![Hardware {
                            number_plate: comms.metadata.number_plate.to_string(),
                            category: "TheBeginnerCar".to_string(),
                            is_connected: current_state,
                        }],
                    };

                    if tx_clone.send(Ok(update_msg)).await.is_err() {
                        break;
                    }
                }
            });
        }

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    type WatchHardwarePoseStream =
        Pin<Box<dyn Stream<Item = Result<WatchHardwarePoseResponse, Status>> + Send + 'static>>;

    async fn watch_hardware_pose(
        &self,
        request: Request<WatchHardwarePoseRequest>,
    ) -> Result<Response<Self::WatchHardwarePoseStream>, Status> {
        dbg!("watch pose");

        let Some(vehicle) = self
            .the_beginner_cars
            .iter()
            .find(|f| f.metadata.number_plate == request.get_ref().number_plate)
        else {
            let status = Status::failed_precondition(format!(
                "the vehicle with number plate {} is not found",
                request.get_ref().number_plate
            ));
            return Err(status);
        };
        let number_plate = vehicle.metadata.number_plate.to_string();

        // TODO: switch to watchstream
        let stream = BroadcastStream::new(vehicle.incoming_tx.subscribe()).filter_map({
            move |result| {
                let number_plate = number_plate.clone();
                async move {
                    match result {
                        Ok(message) => match message {
                            Position {
                                x,
                                y,
                                rotation_radians,
                                timestamp_unix_epoch_ms,
                            } => Some(Ok(WatchHardwarePoseResponse {
                                number_plate,
                                x,
                                y,
                                rotation_radians,
                                timestamp_unix_epoch_ms,
                            })),
                            _ => None,
                        },
                        Err(_) => {
                            // BroadcastStream reports lagged receivers here.
                            // Skip lost messages and continue streaming.
                            None
                        }
                    }
                }
            }
        });

        Ok(Response::new(Box::pin(stream)))
    }

    async fn notify_hardware_coordinates(
        &self,
        request: Request<NotifyHardwareCoordinatesRequest>,
    ) -> Result<Response<NotifyHardwareCoordinatesResponse>, Status> {
        let request = request.into_inner();

        for hardware_coordinate in request.hardware_coordinates {
            let beginner_car = self.the_beginner_cars.iter().find(|the_beginner_car| {
                the_beginner_car.metadata.aruco_marker_id == hardware_coordinate.aruco_marker_id
            });

            if let Some(beginner_car) = beginner_car {
                beginner_car.send(the_beginner_car_tcp_protocol::TheBeginnerCarIncomingMessages::NotifyPositionFromCamera { x: hardware_coordinate.x, y: hardware_coordinate.y, rotation_radians: hardware_coordinate.rotation_radians, timestamp_unix_epoch_ms: hardware_coordinate.timestamp_unix_epoch_ms }).await;
            }
        }
        Ok(Response::new(NotifyHardwareCoordinatesResponse {}))
    }
}
