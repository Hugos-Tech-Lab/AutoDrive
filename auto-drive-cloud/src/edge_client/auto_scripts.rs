use std::{sync::Arc, time::Duration};

use anyhow::bail;
use futures_util::Stream;
use log::{info, warn};
use tokio::{sync::watch, task::JoinHandle, time::timeout};
use tokio_stream::{
    StreamExt,
    wrappers::{WatchStream, errors::BroadcastStreamRecvError},
};
use tonic::transport::Channel;

use crate::{
    auto_drive_edge_service::v1::{
        CancelAutoScriptRequest, EnqueueAutoScriptRequest, WatchAutoScriptsRequest,
        auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
    },
    edge_client::{auto_script::AutoScript, grpc_connect::grpc_connect},
};

#[derive(Debug, Clone)]
pub struct AutoScripts {
    auto_scripts: watch::Receiver<Vec<AutoScript>>,
    watch_auto_scripts_task: Arc<JoinHandle<()>>,
    client: AutoDriveEdgeServiceClient<Channel>,
}

impl AutoScripts {
    pub fn new(client: AutoDriveEdgeServiceClient<Channel>) -> Self {
        let (auto_scripts_tx, auto_scripts) = watch::channel::<Vec<AutoScript>>(vec![]);

        let watch_auto_scripts_task =
            tokio::spawn(watch_auto_scripts(client.clone(), auto_scripts_tx));

        Self {
            auto_scripts,
            watch_auto_scripts_task: Arc::new(watch_auto_scripts_task),
            client,
        }
    }

    pub async fn cancel(&self, run_uuid: String) -> anyhow::Result<()> {
        let valid_run_uuid;
        {
            let auto_scripts = self.auto_scripts.borrow();

            let script = auto_scripts
                .iter()
                .find(|auto_script| auto_script.run_uuid == run_uuid);

            if let Some(_) = script {
                valid_run_uuid = true;
            } else {
                valid_run_uuid = false;
            }
        }

        if !valid_run_uuid {
            bail!("script with UUID ({run_uuid}) not found");
        }

        let response = self
            .client
            .clone()
            .cancel_auto_script(CancelAutoScriptRequest { run_uuid })
            .await; // TODO check response 

        let _response = match response {
            Ok(response) => response,
            Err(err) => bail!("edge cancel request failed with status ({err})"),
        };

        Ok(())
    }

    pub fn stream(&self) -> impl Stream<Item = Vec<AutoScript>> + Send + Sync + 'static {
        WatchStream::new(self.auto_scripts.clone())
    }

    pub async fn find_auto_script(
        &self,
        run_uuid: String,
        timeout_duration: Duration,
    ) -> anyhow::Result<AutoScript> {
        let mut auto_scripts = self.auto_scripts.clone();

        let auto_scripts = timeout(
            timeout_duration,
            auto_scripts.wait_for(|auto_scripts| {
                auto_scripts
                    .iter()
                    .find(|auto_script| auto_script.run_uuid == run_uuid)
                    .is_some()
            }),
        )
        .await;

        let auto_scripts = match auto_scripts {
            Ok(auto_scripts) => auto_scripts,
            Err(_err) => bail!(format!(
                "timeout while waiting for auto script ({run_uuid})"
            )),
        };

        let auto_scripts = match auto_scripts {
            Ok(auto_scripts) => auto_scripts,
            Err(err) => bail!(format!("error ({err}) while waiting for script")),
        };

        let auto_script = auto_scripts
            .iter()
            .find(|auto_script| auto_script.run_uuid == run_uuid);

        let auto_script = match auto_script {
            Some(auto_script) => auto_script,
            None => {
                bail!("waited for script, apparently found but now I'm lost (programmer bug here!)")
            }
        };

        Ok(auto_script.clone())
    }

    pub async fn enqueue(
        &self,
        owner_name: String,
        owner_comment: String,
        data: Vec<u8>,
        script_type: i32,
    ) -> anyhow::Result<String> {
        let response = self
            .client
            .clone()
            .enqueue_auto_script(EnqueueAutoScriptRequest {
                owner_name,
                owner_comment,
                data,
                script_type,
            })
            .await
            .unwrap();

        Ok(response.into_inner().run_uuid)
    }
}

async fn watch_auto_scripts(
    client: AutoDriveEdgeServiceClient<Channel>,
    tracked_auto_scripts: watch::Sender<Vec<AutoScript>>,
) {
    let mut stream = grpc_connect(
        client.clone(),
        WatchAutoScriptsRequest {},
        AutoDriveEdgeServiceClient::watch_auto_scripts,
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

        tracked_auto_scripts.send_modify(|tracked_auto_scripts| {
            let updated_tracked_auto_scripts = response
                .scripts
                .into_iter()
                .map(|new_script| {
                    if let Some(pos) = tracked_auto_scripts
                        .iter()
                        .position(|existing| existing.run_uuid == new_script.run_uuid)
                    {
                        tracked_auto_scripts.swap_remove(pos)
                    } else {
                        AutoScript::new(client.clone(), new_script.run_uuid)
                    }
                })
                .collect();

            *tracked_auto_scripts = updated_tracked_auto_scripts;
        });
    }
    info!("no longer watching auto scripts");
}
