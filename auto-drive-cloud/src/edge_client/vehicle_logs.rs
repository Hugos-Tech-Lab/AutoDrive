use std::sync::Arc;

use log::warn;
use tokio::{
    sync::{RwLock, broadcast},
    task::JoinHandle,
};
use tonic::transport::Channel;

use crate::{
    auto_drive_edge_service::v1::{
        LogMessage, WatchVehicleLogsRequest,
        auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
    },
    edge_client::{Logs, grpc_connect::grpc_connect},
};

pub fn vehicle_logs(
    client: AutoDriveEdgeServiceClient<Channel>,
    number_plate: String,
) -> (Logs, JoinHandle<()>) {
    let history: Arc<RwLock<Vec<LogMessage>>> = Default::default();
    let live = broadcast::Sender::<LogMessage>::new(100);
    let logs = Logs::new(history.clone(), live.subscribe());
    let join_handle = tokio::spawn(watch_vehicle_logs(client, number_plate, history, live));
    (logs, join_handle)
}

async fn watch_vehicle_logs(
    client: AutoDriveEdgeServiceClient<Channel>,
    number_plate: String,
    history: Arc<RwLock<Vec<LogMessage>>>,
    live: broadcast::Sender<LogMessage>,
) {
    let mut stream = grpc_connect(
        client,
        WatchVehicleLogsRequest { number_plate },
        AutoDriveEdgeServiceClient::watch_vehicle_logs,
    )
    .await;

    loop {
        let response = stream.message().await;
        let response = match response {
            Ok(Some(response)) => response,
            Err(err) => {
                warn!("error from the edge: {}", err);
                continue;
            }
            Ok(None) => break,
        };

        let Some(message) = response.log_message else {
            warn!("log message not in message");
            continue;
        };

        let mut history_lock = history.write().await;
        live.send(message.clone());
        history_lock.push(message);
    }
}
