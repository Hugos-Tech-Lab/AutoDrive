use std::time::Duration;

use log::warn;
use tokio::{
    sync::watch,
    task::JoinHandle,
    time::{sleep, timeout},
};
use tonic::transport::Channel;

use crate::{
    auto_drive_edge_service::v1::{
        WatchHardwarePoseRequest, auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
    },
    edge_client::Pose,
};

pub fn watch_vehicle_position(
    client: AutoDriveEdgeServiceClient<Channel>,
    number_plate: String,
) -> (watch::Sender<Option<Pose>>, JoinHandle<()>) {
    let (vehicle_position, _) = watch::channel::<Option<Pose>>(None);

    let join_handle = tokio::spawn({
        let mut client = client.clone();
        let vehicle_position = vehicle_position.clone();

        async move {
            loop {
                let result = timeout(
                    Duration::from_secs(5),
                    client.watch_hardware_pose(WatchHardwarePoseRequest {
                        number_plate: number_plate.clone(),
                    }),
                )
                .await;

                let result = match result {
                    Ok(result) => result,
                    Err(_) => {
                        warn!("Timed out");
                        continue;
                    }
                };

                match result {
                    Ok(response) => {
                        let mut stream = response.into_inner();
                        // TODO: remove unwrap
                        while let Some(result) = stream.message().await.unwrap() {

                            // TODO
                            // vehicle_position.send_replace(result.scripts.unwrap()); // TODO: remove unwrap
                        }
                    }
                    Err(err) => {
                        warn!(
                            "Failed to subscribe to auto script overview: {err}. Retrying in 5s..."
                        );
                    }
                }
                sleep(Duration::from_secs(5)).await;
            }
        }
    });

    (vehicle_position, join_handle)
}
