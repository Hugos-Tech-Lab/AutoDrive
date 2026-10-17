use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};

use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    mpsc, oneshot,
};
use tokio_stream::StreamExt;

use crate::auto_drive_edge_service::v1::{
    AutoScripts, CancelAutoScriptRequest, LogMessage, RunAutoScriptRequest,
    WatchAutoScriptOverviewRequest, WatchAutoScriptRequest, WatchTheBeginnerCarEventsRequest,
    auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
    watch_the_beginner_car_events_response::Event,
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
        response_tx: oneshot::Sender<Receiver<LogMessage>>,
    },
    CancelAutoScript {
        run_uuid: String,
    },
    WatchOverview {
        response_tx: oneshot::Sender<(AutoScripts, Receiver<AutoScripts>)>,
    },
}

#[derive(Debug, Clone)]
pub struct Hardware {
    pub sender: mpsc::Sender<ToEdgeMessage>,
    pub logs: Sender<Event>,
    pub the_beginner_car_auto_scripts: Arc<Mutex<HashMap<String, Sender<LogMessage>>>>, // TODO: maybe this could store an array for history
    pub latest_auto_scripts_overview: Arc<Mutex<AutoScripts>>,
    pub watch_auto_scripts_overview: Sender<AutoScripts>,
}

pub async fn edge_client() -> Result<Hardware, Box<dyn Error>> {
    let mut client = AutoDriveEdgeServiceClient::connect("http://127.0.0.1:50051").await?;

    let (incoming_tx, _) = broadcast::channel::<Event>(100);

    let latest_auto_scripts_overview = Arc::new(Mutex::new(AutoScripts {
        running: vec![],
        queued: vec![],
        stopped: vec![],
    }));
    let the_beginner_car_auto_scripts = Arc::new(Mutex::new(HashMap::new()));

    // TODO: track the task!
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

    let (incoming_auto_scripts_tx, _) = broadcast::channel::<AutoScripts>(10);
    tokio::spawn({
        let incoming_auto_scripts_tx = incoming_auto_scripts_tx.clone();
        let mut client = client.clone();
        let latest_auto_scripts_overview = latest_auto_scripts_overview.clone();
        async move {
            let response = client
                .watch_auto_script_overview(WatchAutoScriptOverviewRequest {})
                .await
                .expect("Failed to subscribe to beginner car events");

            let mut stream = response.into_inner();

            while let Some(result) = stream.next().await {
                match result {
                    Ok(event) => {
                        // Forward the event to all broadcast subscribers.
                        if let Some(event) = event.scripts {
                            let _ = incoming_auto_scripts_tx.send(event.clone());
                            let mut latest_auto_scripts_overview =
                                latest_auto_scripts_overview.lock().unwrap();
                            *latest_auto_scripts_overview = event;
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
    tokio::spawn({
        let latest_auto_scripts_overview = latest_auto_scripts_overview.clone();
        let incoming_auto_scripts_tx = incoming_auto_scripts_tx.clone();
        let the_beginner_car_auto_scripts = the_beginner_car_auto_scripts.clone();
        async move {
            while let Some(value) = receiver.recv().await {
                match value {
                    ToEdgeMessage::StartAutoScript {
                        owner,
                        data,
                        uploader_comment,
                        response_tx,
                    } => {
                        dbg!("starting");
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
                            .insert(run_uuid.clone(), sender.clone());

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
                    ToEdgeMessage::WatchAutoScript {
                        run_uuid,
                        response_tx,
                    } => {
                        let log_sender = the_beginner_car_auto_scripts
                            .lock()
                            .unwrap()
                            .get(&run_uuid)
                            .unwrap()
                            .clone();
                        response_tx.send(log_sender.subscribe()).unwrap();
                    }
                    ToEdgeMessage::WatchOverview { response_tx } => {
                        let latest_auto_scripts_overview =
                            latest_auto_scripts_overview.lock().unwrap().clone();

                        response_tx
                            .send((
                                latest_auto_scripts_overview,
                                incoming_auto_scripts_tx.subscribe(),
                            ))
                            .unwrap();
                    }
                };
            }
        }
    });

    Ok(Hardware {
        logs: incoming_tx,
        sender,
        latest_auto_scripts_overview: latest_auto_scripts_overview,
        watch_auto_scripts_overview: incoming_auto_scripts_tx,
        the_beginner_car_auto_scripts,
    })
}
