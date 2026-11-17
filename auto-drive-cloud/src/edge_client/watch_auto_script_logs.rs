use std::time::Duration;

use log::warn;
use tokio::{
    task::JoinHandle,
    time::{sleep, timeout},
};
use tonic::transport::Channel;

use crate::{
    auto_drive_edge_service::v1::{
        WatchAutoScriptLogsRequest, auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
    },
    edge_client::Logs,
};

pub fn watch_auto_script_logs(
    client: AutoDriveEdgeServiceClient<Channel>,
    run_uuid: String,
) -> (Logs, JoinHandle<()>) {
    let logs = Logs::default();

    let join_handle = tokio::spawn({
        let mut client = client.clone();
        let logs = logs.clone();

        async move {
            loop {
                let result = timeout(
                    Duration::from_secs(5),
                    client.watch_auto_script_logs(WatchAutoScriptLogsRequest {
                        run_uuid: run_uuid.clone(),
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
                                logs.live_tx.send(log_message.clone());
                                let mut history_lock = logs.history.write().await;
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

    (logs, join_handle)
}
