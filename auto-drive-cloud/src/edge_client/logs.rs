use std::sync::Arc;

use futures_util::{
    Stream, StreamExt,
    stream::{self},
};

use log::{info, warn};
use tokio::{
    sync::{
        RwLock,
        broadcast::{self},
    },
    task::JoinHandle,
};
use tokio_stream::wrappers::{BroadcastStream, errors::BroadcastStreamRecvError};
use tonic::transport::Channel;

use crate::{
    auto_drive_edge_service::v1::{
        LogMessage, WatchAutoScriptLogsRequest, WatchVehicleLogsRequest,
        auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
    },
    edge_client::grpc_connect::grpc_connect,
};

#[derive(Debug)]
pub struct Logs {
    history: Arc<RwLock<Vec<LogMessage>>>,
    live: broadcast::Receiver<LogMessage>,
    live_task: Arc<JoinHandle<()>>,
}

impl Clone for Logs {
    fn clone(&self) -> Self {
        Self {
            history: self.history.clone(),
            live: self.live.resubscribe(),
            live_task: self.live_task.clone(),
        }
    }
}

impl Logs {
    pub fn from_vehicle(client: AutoDriveEdgeServiceClient<Channel>, number_plate: String) -> Self {
        let history: Arc<RwLock<Vec<LogMessage>>> = Default::default();
        let live_tx = broadcast::Sender::<LogMessage>::new(100);
        let live = live_tx.subscribe();
        let live_task = tokio::spawn(watch_vehicle_logs(
            client,
            number_plate,
            history.clone(),
            live_tx,
        ));

        Self {
            history,
            live,
            live_task: Arc::new(live_task),
        }
    }

    pub fn from_auto_script(client: AutoDriveEdgeServiceClient<Channel>, run_uuid: String) -> Self {
        let history: Arc<RwLock<Vec<LogMessage>>> = Default::default();
        let live_tx = broadcast::Sender::<LogMessage>::new(100);
        let live = live_tx.subscribe();
        let live_task = tokio::spawn(watch_auto_script_logs(
            client,
            run_uuid,
            history.clone(),
            live_tx,
        ));

        Self {
            history,
            live,
            live_task: Arc::new(live_task),
        }
    }

    pub async fn stream_with_history(
        &self,
    ) -> Option<
        impl Stream<Item = Result<LogMessage, BroadcastStreamRecvError>> + Send + Sync + 'static,
    > {
        let history_guard = self.history.read().await;
        let history = history_guard.clone();
        drop(history_guard);

        let history_stream = stream::iter(history.into_iter().map(Ok));
        let live_stream = BroadcastStream::new(self.live.resubscribe());

        Some(history_stream.chain(live_stream))
    }
}

async fn watch_vehicle_logs(
    client: AutoDriveEdgeServiceClient<Channel>,
    number_plate: String,
    history: Arc<RwLock<Vec<LogMessage>>>,
    live: broadcast::Sender<LogMessage>,
) {
    let mut stream = grpc_connect(
        client,
        WatchVehicleLogsRequest {
            number_plate: number_plate.clone(),
        },
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

    info!("no longer watching logs for vehicle ({number_plate})");
}

async fn watch_auto_script_logs(
    client: AutoDriveEdgeServiceClient<Channel>,
    run_uuid: String,
    history: Arc<RwLock<Vec<LogMessage>>>,
    live: broadcast::Sender<LogMessage>,
) {
    let mut stream = grpc_connect(
        client,
        WatchAutoScriptLogsRequest {
            run_uuid: run_uuid.clone(),
        },
        AutoDriveEdgeServiceClient::watch_auto_script_logs,
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

    info!("no longer watching logs for auto script ({run_uuid})");
}
