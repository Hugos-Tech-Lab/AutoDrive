use std::{pin::Pin, sync::Arc};

use futures_util::{Stream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status};

use crate::{
    auto_drive_edge_service::v1::{
        CancelAutoScriptRequest, CancelAutoScriptResponse, RunAutoScriptRequest,
        RunAutoScriptResponse, WatchAutoScriptQueueRequest, WatchAutoScriptQueueResponse,
        WatchAutoScriptRequest, WatchAutoScriptResponse, WatchTheBeginnerCarEventsRequest,
        WatchTheBeginnerCarEventsResponse, auto_drive_edge_service_server::AutoDriveEdgeService,
    },
    auto_script_manager::{AutoScriptManager, AutoScriptQueueSnapshot},
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
            .enqueue_auto_script(request.owner, request.data)
            .await;

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

    type WatchAutoScriptQueueStream =
        Pin<Box<dyn Stream<Item = Result<WatchAutoScriptQueueResponse, Status>> + Send + 'static>>;

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
            // TODO: Convert your internal `AutoScriptEvents` to `WatchAutoScriptResponse`
            // For example:
            // Ok(WatchAutoScriptResponse { log: event.into() })
            Ok(WatchAutoScriptResponse {})
        });

        let live_stream = BroadcastStream::new(live_rx).filter_map(|result| async move {
            match result {
                Ok(event) => {
                    // TODO: Convert your internal `AutoScriptEvents` to `WatchAutoScriptResponse`
                    Some(Ok(WatchAutoScriptResponse {}))
                }
                Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(skipped)) => {
                    eprintln!("Watch stream lagged; skipped {skipped} messages");
                    None
                }
            }
        });

        // 4. Chain the history directly into the live stream
        let combined_stream = history_stream.chain(live_stream);

        Ok(Response::new(Box::pin(combined_stream)))
    }

    async fn watch_auto_script_queue(
        &self,
        _request: Request<WatchAutoScriptQueueRequest>,
    ) -> Result<Response<Self::WatchAutoScriptQueueStream>, Status> {
        let (snapshot, queue_receiver) = self.auto_script_manager.watch_auto_script_queue().await;

        let current_snapshot = tokio_stream::once(Ok(WatchAutoScriptQueueResponse {
            scripts: Some(snapshot.into()),
        }));

        let live_stream = BroadcastStream::new(queue_receiver).filter_map(|result| async move {
            match result {
                Ok(snapshot) => Some(Ok(WatchAutoScriptQueueResponse {
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
}
