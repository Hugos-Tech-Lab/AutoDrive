use std::pin::Pin;

use futures_util::{Stream, StreamExt};
use log::warn;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use auto_drive_edge_protocol::{
    auto_drive_edge_service::v1::{
        AutoScriptMetadata, CancelAutoScriptRequest, CancelAutoScriptResponse,
        EnqueueAutoScriptRequest, EnqueueAutoScriptResponse, GetVehiclesRequest,
        GetVehiclesResponse, NotifyHardwareCoordinatesRequest, NotifyHardwareCoordinatesResponse,
        RunAutoScriptRequest, RunAutoScriptResponse, WatchAutoScriptLogsRequest,
        WatchAutoScriptLogsResponse, WatchAutoScriptsRequest, WatchAutoScriptsResponse,
        WatchHardwarePoseRequest, WatchHardwarePoseResponse, WatchHardwareStatusRequest,
        WatchHardwareStatusResponse, WatchVehicleLogsRequest, WatchVehicleLogsResponse,
        auto_drive_edge_service_server::AutoDriveEdgeService,
    },
    auto_script_permission_level::AutoScriptPermissionLevel,
};

use crate::{auto_scripts::AutoScripts, vehicle::the_beginner_car::TheBeginnerCar};

pub struct GrpcService {
    auto_scripts: AutoScripts,
    the_beginner_cars: Vec<TheBeginnerCar>,
}

impl GrpcService {
    pub fn new(auto_scripts: AutoScripts, the_beginner_cars: Vec<TheBeginnerCar>) -> Self {
        Self {
            auto_scripts,
            the_beginner_cars,
        }
    }
}
#[tonic::async_trait]
impl AutoDriveEdgeService for GrpcService {
    async fn get_vehicles(
        &self,
        _request: Request<GetVehiclesRequest>,
    ) -> Result<Response<GetVehiclesResponse>, Status> {
        let number_plates = self
            .the_beginner_cars
            .iter()
            .map(|vehicle| vehicle.number_plate())
            .collect();
        Ok(Response::new(GetVehiclesResponse { number_plates }))
    }

    type WatchAutoScriptsStream = std::pin::Pin<
        Box<
            dyn futures_util::Stream<Item = Result<WatchAutoScriptsResponse, Status>>
                + Send
                + 'static,
        >,
    >;

    async fn watch_auto_scripts(
        &self,
        _request: Request<WatchAutoScriptsRequest>,
    ) -> Result<Response<Self::WatchAutoScriptsStream>, Status> {
        let stream = self.auto_scripts.stream();
        let scripts = stream.map(|auto_scripts| {
            let scripts: Vec<AutoScriptMetadata> = auto_scripts
                .iter()
                .map(|auto_script| auto_script.metadata.clone().into())
                .collect();

            Ok(WatchAutoScriptsResponse { scripts })
        });

        Ok(Response::new(Box::pin(scripts)))
    }

    async fn enqueue_auto_script(
        &self,
        request: Request<EnqueueAutoScriptRequest>,
    ) -> Result<Response<EnqueueAutoScriptResponse>, Status> {
        let request = request.into_inner();

        let run_uuid = self.auto_scripts.enqueue_auto_script(
            request.owner_name,
            request.owner_comment,
            request.data,
            AutoScriptPermissionLevel::Standard,
        );
        Ok(Response::new(EnqueueAutoScriptResponse { run_uuid }))
    }

    async fn run_auto_script(
        &self,
        request: Request<RunAutoScriptRequest>,
    ) -> Result<Response<RunAutoScriptResponse>, Status> {
        let request = request.into_inner();

        let vehicle = self
            .the_beginner_cars
            .iter()
            .find(|vehicle| vehicle.number_plate() == request.number_plate);

        let vehicle = vehicle.unwrap();

        if let Err(_) = vehicle.start_auto_script(request.run_uuid).await {
            warn!("error! TODO: improve this error ");
        };

        Ok(Response::new(RunAutoScriptResponse {}))
    }

    async fn cancel_auto_script(
        &self,
        request: Request<CancelAutoScriptRequest>,
    ) -> Result<Response<CancelAutoScriptResponse>, Status> {
        let request = request.into_inner();
        self.auto_scripts.cancel(request.run_uuid).await;
        Ok(Response::new(CancelAutoScriptResponse {}))
    }

    type WatchAutoScriptLogsStream =
        Pin<Box<dyn Stream<Item = Result<WatchAutoScriptLogsResponse, Status>> + Send + 'static>>;

    async fn watch_auto_script_logs(
        &self,
        request: Request<WatchAutoScriptLogsRequest>,
    ) -> Result<Response<Self::WatchAutoScriptLogsStream>, Status> {
        let request = request.into_inner();
        let auto_script = self
            .auto_scripts
            .auto_script(request.run_uuid.clone())
            .await;
        let Some(auto_script) = auto_script else {
            return Err(Status::not_found(format!(
                "AutoScript '{}' does not exist",
                request.run_uuid
            )));
        };

        let stream = auto_script.payload.logs.stream_with_history().await;
        let scripts = stream.map(|log| {
            let log_message = match log {
                Ok(log) => log,
                Err(err) => {
                    return Err(Status::not_found(format!(
                        "Broadcast stream error: ({})",
                        err
                    )));
                }
            };

            Ok(WatchAutoScriptLogsResponse {
                log_message: Some(log_message.into()),
            })
        });

        Ok(Response::new(Box::pin(scripts)))
    }

    type WatchVehicleLogsStream = ReceiverStream<Result<WatchVehicleLogsResponse, Status>>;

    async fn watch_vehicle_logs(
        &self,
        _request: Request<WatchVehicleLogsRequest>,
    ) -> Result<Response<Self::WatchVehicleLogsStream>, Status> {
        return Err(Status::not_found(format!("NOT IMPLEMENTED",)));
    }

    type WatchHardwareStatusStream = ReceiverStream<Result<WatchHardwareStatusResponse, Status>>;

    async fn watch_hardware_status(
        &self,
        _request: Request<WatchHardwareStatusRequest>,
    ) -> Result<Response<Self::WatchHardwareStatusStream>, Status> {
        return Err(Status::not_found(format!("NOT IMPLEMENTED",)));
    }

    type WatchHardwarePoseStream = ReceiverStream<Result<WatchHardwarePoseResponse, Status>>;

    async fn watch_hardware_pose(
        &self,
        _request: Request<WatchHardwarePoseRequest>,
    ) -> Result<Response<Self::WatchHardwarePoseStream>, Status> {
        return Err(Status::not_found(format!("NOT IMPLEMENTED",)));
    }

    async fn notify_hardware_coordinates(
        &self,
        _request: Request<NotifyHardwareCoordinatesRequest>,
    ) -> Result<Response<NotifyHardwareCoordinatesResponse>, Status> {
        return Err(Status::not_found(format!("NOT IMPLEMENTED",)));
    }
}
