use std::{sync::Arc, time::Duration};

use log::warn;
use tokio::{
    sync::{RwLock, broadcast},
    task::JoinHandle,
    time::{sleep, timeout},
};
use tonic::transport::Channel;

use crate::{
    auto_drive_edge_service::v1::{
        LogMessage, WatchVehicleLogsRequest,
        auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
    },
    edge_client::Logs,
};

pub fn watch_vehicle_logs(
    client: AutoDriveEdgeServiceClient<Channel>,
    number_plate: String,
) -> (Logs, JoinHandle<()>) {
    let history: Arc<RwLock<Vec<LogMessage>>> = Default::default();

    let live_tx = broadcast::Sender::<LogMessage>::new(100);
    let live = live_tx.subscribe();

    let join_handle = tokio::spawn({
        let mut client = client.clone();
        let history = history.clone();

        async move {
            loop {
                let result = timeout(
                    Duration::from_secs(5),
                    client.watch_vehicle_logs(WatchVehicleLogsRequest {
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
                        while let Some(logs_response) = stream.message().await.unwrap() {
                            if let Some(log_message) = logs_response.log_message {
                                let mut history_lock = history.write().await;
                                live_tx.send(log_message.clone());
                                history_lock.push(log_message);
                            };
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

    let logs = Logs::new(history, live);
    (logs, join_handle)
}
