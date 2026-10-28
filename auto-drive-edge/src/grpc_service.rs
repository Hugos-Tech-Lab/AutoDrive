use std::{pin::Pin, sync::Arc};

use futures_util::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::{BroadcastStream, ReceiverStream};
use tonic::{Request, Response, Status};

use crate::{
    auto_drive_edge_service::v1::{
        CancelAutoScriptRequest, CancelAutoScriptResponse, Hardware, RunAutoScriptRequest,
        RunAutoScriptResponse, WatchAutoScriptOverviewRequest, WatchAutoScriptOverviewResponse,
        WatchAutoScriptRequest, WatchAutoScriptResponse, WatchHardwareStatusRequest,
        WatchHardwareStatusResponse, WatchTheBeginnerCarEventsRequest,
        WatchTheBeginnerCarEventsResponse, auto_drive_edge_service_server::AutoDriveEdgeService,
    },
    auto_script_manager::AutoScriptManager,
    grpc_service::types::to_grpc_log_message,
    the_beginner_car_comms::TheBeginnerCarComms,
};

pub mod types;

pub struct GrpcService {
    auto_script_manager: Arc<AutoScriptManager>,
    the_beginner_cars: Vec<TheBeginnerCarComms>,
}

impl GrpcService {
    pub fn new(
        auto_script_manager: Arc<AutoScriptManager>,
        the_beginner_cars: Vec<TheBeginnerCarComms>,
    ) -> Self {
        Self {
            auto_script_manager,
            the_beginner_cars,
        }
    }
}

#[tonic::async_trait]
impl AutoDriveEdgeService for GrpcService {
    async fn run_auto_script(
        &self,
        request: Request<RunAutoScriptRequest>,
    ) -> Result<Response<RunAutoScriptResponse>, Status> {
        let request = request.into_inner();
        let run_uuid = self
            .auto_script_manager
            .enqueue_auto_script(request.owner_name, request.owner_comment, request.data)
            .await;
        dbg!("run request");

        Ok(Response::new(RunAutoScriptResponse { run_uuid }))
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

    type WatchAutoScriptStream =
        Pin<Box<dyn Stream<Item = Result<WatchAutoScriptResponse, Status>> + Send + 'static>>;

    type WatchAutoScriptOverviewStream = Pin<
        Box<dyn Stream<Item = Result<WatchAutoScriptOverviewResponse, Status>> + Send + 'static>,
    >;

    type WatchHardwareStatusStream =
        Pin<Box<dyn Stream<Item = Result<WatchHardwareStatusResponse, Status>> + Send + 'static>>;

    async fn watch_auto_script(
        &self,
        request: Request<WatchAutoScriptRequest>,
    ) -> Result<Response<Self::WatchAutoScriptStream>, Status> {
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

    async fn watch_auto_script_overview(
        &self,
        _request: Request<WatchAutoScriptOverviewRequest>,
    ) -> Result<Response<Self::WatchAutoScriptOverviewStream>, Status> {
        let (snapshot, queue_receiver) =
            self.auto_script_manager.watch_auto_script_overview().await;

        let current_snapshot = tokio_stream::once(Ok(WatchAutoScriptOverviewResponse {
            scripts: Some(snapshot.into()),
        }));

        let live_stream = BroadcastStream::new(queue_receiver).filter_map(|result| async move {
            match result {
                Ok(snapshot) => Some(Ok(WatchAutoScriptOverviewResponse {
                    scripts: Some(snapshot.into()),
                })),
                Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(skipped)) => {
                    eprintln!("Watch stream lagged; skipped {skipped} messages");
                    None
                }
            }
        });

        let scripts = current_snapshot.chain(live_stream);

        Ok(Response::new(Box::pin(scripts)))
    }

    type WatchTheBeginnerCarEventsStream = Pin<
        Box<dyn Stream<Item = Result<WatchTheBeginnerCarEventsResponse, Status>> + Send + 'static>,
    >;

    async fn watch_the_beginner_car_events(
        &self,
        request: Request<WatchTheBeginnerCarEventsRequest>,
    ) -> Result<Response<Self::WatchTheBeginnerCarEventsStream>, Status> {
        // TODO: find the ID from the request
        let receiver = self.the_beginner_cars.first().unwrap();

        let stream = BroadcastStream::new(receiver.incoming_tx.subscribe()).filter_map(
            |result| async move {
                match result {
                    Ok(message) => Some(Ok(message.into())),
                    Err(_) => {
                        // BroadcastStream reports lagged receivers here.
                        // Skip lost messages and continue streaming.
                        None
                    }
                }
            },
        );

        Ok(Response::new(Box::pin(stream)))
    }

    async fn watch_hardware_status(
        &self,
        _request: Request<WatchHardwareStatusRequest>,
    ) -> Result<Response<Self::WatchHardwareStatusStream>, Status> {
        let (tx, rx) = mpsc::channel(128);

        let mut initial_hardware = Vec::new();

        for car_entry in self.the_beginner_cars.iter() {
            let comms = car_entry;

            initial_hardware.push(Hardware {
                number_plate: comms.number_plate.clone(),
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
                            number_plate: comms.number_plate.clone(),
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
}
