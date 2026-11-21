use std::sync::Arc;

use log::{info, warn};
use tokio::{sync::watch, task::JoinHandle};
use tonic::transport::Channel;

use crate::{
    auto_drive_edge_service::v1::{
        AutoScripts, WatchAutoScriptOverviewRequest,
        auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
    },
    edge_client::grpc_connect::grpc_connect,
};

#[derive(Debug, Clone)]
pub struct AutoScriptsOverview {
    live: watch::Receiver<AutoScripts>,
    live_task: Arc<JoinHandle<()>>,
}

impl AutoScriptsOverview {
    pub fn new(client: AutoDriveEdgeServiceClient<Channel>) -> Self {
        let (scripts_overview, scripts_overview_rx) = watch::channel::<AutoScripts>(AutoScripts {
            running: vec![],
            queued: vec![],
            stopped: vec![],
        });

        let join_handle = tokio::spawn(watch_auto_scripts_overview(client, scripts_overview));

        Self {
            live: scripts_overview_rx,
            live_task: Arc::new(join_handle),
        }
    }
}

async fn watch_auto_scripts_overview(
    client: AutoDriveEdgeServiceClient<Channel>,
    scripts_overview: watch::Sender<AutoScripts>,
) {
    let mut stream = grpc_connect(
        client,
        WatchAutoScriptOverviewRequest {},
        AutoDriveEdgeServiceClient::watch_auto_script_overview,
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

        let Some(scripts) = response.scripts else {
            warn!("scripts not in message");
            continue;
        };

        scripts_overview.send_replace(scripts);
    }
    info!("no longer watching auto scripts overview");
}
