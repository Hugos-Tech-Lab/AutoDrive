use std::sync::Arc;

use log::{info, warn};
use tokio::{sync::watch, task::JoinHandle};
use tonic::transport::Channel;

use crate::{
    auto_drive_edge_service::v1::{
        WatchQueuedAutoScriptsRequest, WatchRunningAutoScriptsRequest,
        auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
    },
    edge_client::{auto_script::AutoScript, grpc_connect::grpc_connect},
};

#[derive(Debug, Clone)]
pub struct AutoScripts {
    running_auto_scripts: watch::Receiver<Vec<AutoScript>>,
    running_auto_scripts_task: Arc<JoinHandle<()>>,
    queued_auto_scripts: watch::Receiver<Vec<AutoScript>>,
    queued_auto_scripts_task: Arc<JoinHandle<()>>,
    stopped_auto_scripts: watch::Receiver<Vec<AutoScript>>,
    stopped_auto_scripts_task: Arc<JoinHandle<()>>,
}

impl AutoScripts {
    pub fn new(client: AutoDriveEdgeServiceClient<Channel>) -> Self {
        let (running_auto_scripts_tx, running_auto_scripts) =
            watch::channel::<Vec<AutoScript>>(vec![]);

        let watch_running_auto_scripts_task = tokio::spawn(watch_running_auto_scripts(
            client.clone(),
            running_auto_scripts_tx,
        ));

        let (queued_auto_scripts_tx, queued_auto_scripts) =
            watch::channel::<Vec<AutoScript>>(vec![]);

        let watch_queued_auto_scripts_task = tokio::spawn(watch_running_auto_scripts(
            client.clone(),
            queued_auto_scripts_tx,
        ));

        Self {
            running_auto_scripts,
            running_auto_scripts_task: Arc::new(watch_running_auto_scripts_task),
            queued_auto_scripts,
            queued_auto_scripts_task: Arc::new(watch_queued_auto_scripts_task),
            stopped_auto_scripts: todo!(),
            stopped_auto_scripts_task: todo!(),
        }
    }
}

async fn watch_running_auto_scripts(
    client: AutoDriveEdgeServiceClient<Channel>,
    running_scripts: watch::Sender<Vec<AutoScript>>,
) {
    let mut stream = grpc_connect(
        client.clone(),
        WatchRunningAutoScriptsRequest {},
        AutoDriveEdgeServiceClient::watch_running_auto_scripts,
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

        running_scripts.send_modify(|auto_scripts| {
            let updated_scripts = response
                .scripts
                .into_iter() // use .iter().cloned() if `response` is borrowed
                .map(|new_script| {
                    if let Some(pos) = auto_scripts
                        .iter()
                        .position(|existing| existing.run_uuid == new_script.run_uuid)
                    {
                        auto_scripts.swap_remove(pos)
                    } else {
                        AutoScript::new(client.clone(), new_script.run_uuid)
                    }
                })
                .collect();

            *auto_scripts = updated_scripts;
        });
    }
    info!("no longer watching auto scripts overview");
}

async fn watch_queued_auto_scripts(
    client: AutoDriveEdgeServiceClient<Channel>,
    queued_scripts: watch::Sender<Vec<AutoScript>>,
) {
    let mut stream = grpc_connect(
        client.clone(),
        WatchQueuedAutoScriptsRequest {},
        AutoDriveEdgeServiceClient::watch_queued_auto_scripts,
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

        queued_scripts.send_modify(|auto_scripts| {
            let updated_scripts = response
                .scripts
                .into_iter() // use .iter().cloned() if `response` is borrowed
                .map(|new_script| {
                    if let Some(pos) = auto_scripts
                        .iter()
                        .position(|existing| existing.run_uuid == new_script.run_uuid)
                    {
                        auto_scripts.swap_remove(pos)
                    } else {
                        AutoScript::new(client.clone(), new_script.run_uuid)
                    }
                })
                .collect();

            *auto_scripts = updated_scripts;
        });
    }
    info!("no longer watching auto scripts overview");
}
