use std::{pin::Pin, sync::Arc};

use futures_util::{Stream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status};

use crate::{
    auto_drive_edge_service::v1::{
        CancelAutoScriptRequest,
        CancelAutoScriptResponse,
        RunAutoScriptRequest,
        RunAutoScriptResponse,
        WatchAutoScriptQueueRequest,
        WatchAutoScriptQueueResponse,
        WatchAutoScriptRequest,
        WatchAutoScriptResponse, // Added missing imports
        WatchTheBeginnerCarEventsRequest,
        WatchTheBeginnerCarEventsResponse,
        auto_drive_edge_service_server::AutoDriveEdgeService,
    },
    auto_script_manager::AutoScriptManager,
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
        let run_uuid = self
            .auto_script_manager
            .enqueue_auto_script(request.into_inner().data)
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

    // FIXED: Added missing method required by the Proto trait
    async fn watch_auto_script(
        &self,
        _request: Request<WatchAutoScriptRequest>,
    ) -> Result<Response<WatchAutoScriptResponse>, Status> {
        // Implementation goes here
        Ok(Response::new(WatchAutoScriptResponse {}))
    }

    async fn watch_auto_script_queue(
        &self,
        _request: Request<WatchAutoScriptQueueRequest>,
    ) -> Result<Response<WatchAutoScriptQueueResponse>, Status> {
        // Implementation goes here
        Ok(Response::new(WatchAutoScriptQueueResponse {}))
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
