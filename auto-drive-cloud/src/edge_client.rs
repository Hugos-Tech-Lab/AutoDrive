use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};

use tokio::sync::{
    broadcast::{self, Receiver, Sender}, mpsc, oneshot,
};
use tokio_stream::StreamExt;
use tonic::Streaming;

use crate::{
    auto_drive_edge_service::v1::{
        CancelAutoScriptRequest, LogMessage, RunAutoScriptRequest, WatchAutoScriptRequest,
        WatchTheBeginnerCarEventsRequest,
        auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
        watch_the_beginner_car_events_response::Event,
    }
};

pub enum ToEdgeMessage {
    StartAutoScript {
        owner: String,
        data: Vec<u8>,
        uploader_comment: String,
        response_tx: oneshot::Sender<String>,
    },
    WatchAutoScript {
        run_uuid: String,
        response_tx: oneshot::Sender<Receiver<LogMessage>>
    },
    CancelAutoScript {
        run_uuid: String,
    },
}

#[derive(Debug, Clone)]
struct AutoScriptSnapshot {
    running_scripts: (),
    queued_scripts: (),
    stopped_scripts: (),
}

#[derive(Debug, Clone)]
pub struct Hardware {
    pub sender: mpsc::Sender<ToEdgeMessage>,
    pub logs: Sender<Event>,
    pub the_beginner_car_auto_scripts: Arc<Mutex<HashMap<String, Sender<LogMessage>>>>, // TODO: maybe this could store an array for history
    pub latest_scripts: AutoScriptSnapshot,
    pub watch_scripts: Sender<AutoScriptSnapshot>,
}

pub async fn edge_client() -> Result<Hardware, Box<dyn Error>> {
    let mut client = AutoDriveEdgeServiceClient::connect("http://127.0.0.1:50051").await?;

    let (incoming_tx, _) = broadcast::channel::<Event>(100);

    let the_beginner_car_auto_scripts = Arc::new(Mutex::new(HashMap::new()));

    tokio::spawn({
        let incoming_tx = incoming_tx.clone();
        let mut client = client.clone();
        async move {
            let response = client
                .watch_the_beginner_car_events(WatchTheBeginnerCarEventsRequest { build: 1 })
                .await
                .expect("Failed to subscribe to beginner car events");

            let mut stream = response.into_inner();

            while let Some(result) = stream.next().await {
                match result {
                    Ok(event) => {
                        // Forward the event to all broadcast subscribers.
                        if let Some(event) = event.event {
                            let _ = incoming_tx.send(event);
                        }
                    }
                    Err(err) => {
                        eprintln!("Error receiving beginner car event: {err}");
                        break;
                    }
                }
            }
        }
    });

    let (sender, mut receiver) = mpsc::channel::<ToEdgeMessage>(10);
    tokio::spawn(async move {
        while let Some(value) = receiver.recv().await {
            match value {
                ToEdgeMessage::StartAutoScript {
                    owner,
                    data,
                    uploader_comment,
                    response_tx,
                } => {
                    let response = client
                        .run_auto_script(RunAutoScriptRequest {
                            owner,
                            build: 1,
                            data,
                        })
                        .await
                        .unwrap();
                    let response = response.into_inner();
                    let run_uuid = response.run_uuid;

                    let (sender, _) = broadcast::channel(100);

                    the_beginner_car_auto_scripts
                        .lock()
                        .unwrap()
                        .insert("key".to_string(), sender.clone());

                    let watch_auto_script = client
                        .watch_auto_script(WatchAutoScriptRequest {
                            run_uuid: run_uuid.clone(),
                        })
                        .await
                        .unwrap();

                    let mut stream = watch_auto_script.into_inner();

                    // TODO: store this task somewhere
                    tokio::spawn(async move {
                        while let Ok(Some(update)) = stream.message().await {
                            if let Some(log_message) = update.log_message {
                                let _ = sender.send(log_message);
                            }
                        }
                    });

                    response_tx.send(run_uuid.clone()).unwrap();
                }
                ToEdgeMessage::CancelAutoScript { run_uuid } => {
                    client
                        .cancel_auto_script(CancelAutoScriptRequest { run_uuid })
                        .await
                        .unwrap();
                }
                ToEdgeMessage::WatchAutoScript { run_uuid, response_tx } => {
                    let log_sender = the_beginner_car_auto_scripts
                        .lock()
                        .unwrap()
                        .get(&"key".to_string()).unwrap().clone();
                    response_tx.send(log_sender.subscribe()).unwrap();
                }
            };
        }
    });

    Ok(Hardware {
        logs: incoming_tx,
        sender,
        latest_scripts: todo!(),
        watch_scripts: todo!(),
        the_beginner_car_auto_scripts,
    })
}
