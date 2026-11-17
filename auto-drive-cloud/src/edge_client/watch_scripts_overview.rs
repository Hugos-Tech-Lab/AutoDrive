use std::time::Duration;

use log::warn;
use tokio::{
    sync::watch,
    task::JoinHandle,
    time::{sleep, timeout},
};
use tonic::transport::Channel;

use crate::auto_drive_edge_service::v1::{
    AutoScripts, WatchAutoScriptOverviewRequest,
    auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
};

pub fn watch_scripts_overview(
    client: AutoDriveEdgeServiceClient<Channel>,
) -> (watch::Receiver<AutoScripts>, JoinHandle<()>) {
    let (scripts_overview, _) = watch::channel::<AutoScripts>(AutoScripts {
        running: vec![],
        queued: vec![],
        stopped: vec![],
    });

    let join_handle = tokio::spawn({
        let mut client = client.clone();
        let scripts_overview = scripts_overview.clone();
        async move {
            loop {
                let result = timeout(
                    Duration::from_secs(5),
                    client.watch_auto_script_overview(WatchAutoScriptOverviewRequest {}),
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
                            scripts_overview.send_replace(result.scripts.unwrap()); // TODO: remove unwrap
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

    (scripts_overview.subscribe(), join_handle)
}
