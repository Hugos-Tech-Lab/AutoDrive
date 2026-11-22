use std::sync::Arc;

use log::{info, warn};
use tokio::{sync::watch, task::JoinHandle};
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

    pub fn cancel(&self, run_uuid: String) {
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
            return; // TODO: error return because not valid uuid
        }

        // TODO: already release the borrow above
        self.client
            .clone()
            .cancel_auto_script(CancelAutoScriptRequest { run_uuid }); // TODO check response
    }

    pub async fn enqueue(
        &self,
        owner_name: String,
        owner_comment: String,
        data: Vec<u8>,
        script_type: i32,
    ) {
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

        // response.into_inner().run_uuid

        // self.queued_auto_scripts
        //     .scripts
        //     .into_iter() // use .iter().cloned() if `response` is borrowed
        //     .map(|new_script| {
        //         if let Some(pos) = auto_scripts
        //             .iter()
        //             .position(|existing| existing.run_uuid == new_script.run_uuid)
        //         {
        //             auto_scripts.swap_remove(pos)
        //         } else {
        //             AutoScript::new(client.clone(), new_script.run_uuid)
        //         }
        //     })
        //     .collect();

        // response.
    }
}

async fn watch_auto_scripts(
    client: AutoDriveEdgeServiceClient<Channel>,
    auto_scripts: watch::Sender<Vec<AutoScript>>,
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

        auto_scripts.send_modify(|auto_scripts| {
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
    info!("no longer watching auto scripts");
}
