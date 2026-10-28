use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::{
    RwLock,
    broadcast::{self, Receiver, Sender},
    mpsc, oneshot, watch,
};
use tokio::time::sleep;
use tokio_stream::StreamExt;

use crate::auto_drive_edge_service::v1::{
    AutoScripts, CancelAutoScriptRequest, LogMessage, RunAutoScriptRequest,
    WatchAutoScriptOverviewRequest, WatchAutoScriptRequest, WatchHardwareStatusRequest,
    WatchTheBeginnerCarEventsRequest, auto_drive_edge_service_client::AutoDriveEdgeServiceClient,
    watch_the_beginner_car_events_response::Event,
};

type HardwareHashMap = HashMap<String, HardwareDeviceState>;

pub enum StartAutoScriptResponse {
    Uuid { uuid: String },
    EdgeClientUnavailable,
}

pub enum ToEdgeMessage {
    StartAutoScript {
        owner_name: String,
        owner_comment: String,
        data: Vec<u8>,
        response_tx: oneshot::Sender<StartAutoScriptResponse>,
    },
    WatchAutoScript {
        run_uuid: String,
        response_tx: oneshot::Sender<mpsc::Receiver<AutoScriptEvent>>,
    },
    CancelAutoScript {
        run_uuid: String,
    },
    WatchOverview {
        response_tx: oneshot::Sender<(AutoScripts, Receiver<AutoScripts>)>,
    },
    WatchHardware {
        response_tx: oneshot::Sender<(HardwareHashMap, watch::Receiver<HardwareHashMap>)>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDeviceState {
    pub number_plate: String,
    pub category: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone)]
pub enum AutoScriptEvent {
    LogMessage { log_message: LogMessage },
    Exited,
}

#[derive(Debug, Clone)]
pub struct AutoScriptEvents {
    history: Arc<RwLock<Vec<AutoScriptEvent>>>,
    live: Sender<AutoScriptEvent>,
}

#[derive(Debug, Clone)]
pub struct Hardware {
    pub sender: mpsc::Sender<ToEdgeMessage>,
    pub logs: Sender<Event>,
    pub the_beginner_car_auto_script_events: Arc<Mutex<HashMap<String, AutoScriptEvents>>>,
    pub latest_auto_scripts_overview: Arc<Mutex<AutoScripts>>,
    pub watch_auto_scripts_overview: Sender<AutoScripts>,
    pub hardware_states_tx: watch::Sender<HardwareHashMap>,
}

pub async fn edge_client() -> Result<Hardware, Box<dyn Error>> {
    let mut client = loop {
        #[cfg(feature = "development")]
        let auto_drive_edge_ip = "http://127.0.0.1:50051";
        #[cfg(feature = "production")]
        let auto_drive_edge_ip = "http://100.96.0.1:50051";

        match AutoDriveEdgeServiceClient::connect(auto_drive_edge_ip).await {
            Ok(c) => {
                info!("Successfully connected to edge service.");
                break c;
            }
            Err(err) => {
                warn!("Failed to connect to edge service: {err}. Retrying in 5s...");
                sleep(Duration::from_secs(5)).await;
            }
        }
    };

    let (incoming_tx, _) = broadcast::channel::<Event>(100);
    let (hardware_states_tx, _) = watch::channel::<HardwareHashMap>(HashMap::new());

    let latest_auto_scripts_overview = Arc::new(Mutex::new(AutoScripts {
        running: vec![],
        queued: vec![],
        stopped: vec![],
    }));
    let the_beginner_car_auto_script_events: Arc<Mutex<HashMap<String, AutoScriptEvents>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // The loop catches it, sleeps for 5s, and re-requests the stream (which triggers a reconnect).
    tokio::spawn({
        let incoming_tx = incoming_tx.clone();
        let mut client = client.clone();

        async move {
            loop {
                match client
                    .watch_the_beginner_car_events(WatchTheBeginnerCarEventsRequest { build: 1 })
                    .await
                {
                    Ok(response) => {
                        let mut stream = response.into_inner();
                        while let Some(result) = stream.next().await {
                            match result {
                                Ok(event) => {
                                    if let Some(event_data) = event.event {
                                        let _ = incoming_tx.send(event_data.clone());
                                    }
                                }
                                Err(err) => {
                                    warn!(
                                        "Beginner car event stream disconnected: {err}. Reconnecting..."
                                    );
                                    break;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        warn!(
                            "Failed to subscribe to beginner car events: {err}. Retrying in 5s..."
                        );
                    }
                }
                sleep(Duration::from_secs(5)).await;
            }
        }
    });

    let (incoming_auto_scripts_tx, _) = broadcast::channel::<AutoScripts>(10);
    tokio::spawn({
        let incoming_auto_scripts_tx = incoming_auto_scripts_tx.clone();
        let mut client = client.clone();
        let latest_auto_scripts_overview = latest_auto_scripts_overview.clone();

        async move {
            loop {
                match client
                    .watch_auto_script_overview(WatchAutoScriptOverviewRequest {})
                    .await
                {
                    Ok(response) => {
                        let mut stream = response.into_inner();
                        while let Some(result) = stream.next().await {
                            match result {
                                Ok(event) => {
                                    if let Some(event_scripts) = event.scripts {
                                        let _ =
                                            incoming_auto_scripts_tx.send(event_scripts.clone());
                                        let mut latest_overview =
                                            latest_auto_scripts_overview.lock().unwrap();
                                        *latest_overview = event_scripts;
                                    }
                                }
                                Err(err) => {
                                    warn!(
                                        "Auto script overview stream disconnected: {err}. Reconnecting..."
                                    );
                                    break;
                                }
                            }
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

    tokio::spawn({
        let hardware_states_tx = hardware_states_tx.clone();
        let mut client = client.clone();

        async move {
            loop {
                match client
                    .watch_hardware_status(WatchHardwareStatusRequest {})
                    .await
                {
                    Ok(response) => {
                        let mut stream = response.into_inner();
                        while let Some(result) = stream.next().await {
                            match result {
                                Ok(event) => {
                                    hardware_states_tx.send_modify(|v| {
                                        for hardware in event.hardware {
                                            let entry = v
                                                .entry(hardware.number_plate.clone())
                                                .or_insert(HardwareDeviceState {
                                                    number_plate: hardware.number_plate,
                                                    category: hardware.category,
                                                    is_connected: hardware.is_connected,
                                                });
                                            entry.is_connected = hardware.is_connected;
                                        }
                                    });
                                }
                                Err(err) => {
                                    warn!(
                                        "Hardware status stream disconnected: {err}. Reconnecting..."
                                    );
                                    break;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        warn!("Failed to subscribe to hardware status: {err}. Retrying in 5s...");
                    }
                }
                sleep(Duration::from_secs(5)).await;
            }
        }
    });

    let (sender, mut receiver) = mpsc::channel::<ToEdgeMessage>(10);
    tokio::spawn({
        let latest_auto_scripts_overview = latest_auto_scripts_overview.clone();
        let incoming_auto_scripts_tx = incoming_auto_scripts_tx.clone();
        let the_beginner_car_auto_script_events = the_beginner_car_auto_script_events.clone();
        let hardware_states_tx = hardware_states_tx.clone();
        async move {
            while let Some(value) = receiver.recv().await {
                match value {
                    ToEdgeMessage::StartAutoScript {
                        owner_name,
                        owner_comment,
                        data,
                        response_tx,
                    } => {
                        if let Ok(response) = client
                            .run_auto_script(RunAutoScriptRequest {
                                owner_name,
                                owner_comment,
                                build: 1,
                                data,
                            })
                            .await
                        {
                            let response = response.into_inner();
                            let run_uuid = response.run_uuid;

                            let (sender, _) = broadcast::channel(100);
                            // sender.subscribe();

                            let history = Arc::new(RwLock::new(Vec::new()));

                            the_beginner_car_auto_script_events.lock().unwrap().insert(
                                run_uuid.clone(),
                                AutoScriptEvents {
                                    history: history.clone(),
                                    live: sender.clone(),
                                },
                            );
                            if let Ok(watch_auto_script) = client
                                .watch_auto_script(WatchAutoScriptRequest {
                                    run_uuid: run_uuid.clone(),
                                })
                                .await
                            {
                                let mut stream = watch_auto_script.into_inner();

                                tokio::spawn(async move {
                                    while let Ok(Some(update)) = stream.message().await {
                                        if let Some(event) = update.event {
                                            let event =
                                            match event {
                                                crate::auto_drive_edge_service::v1::watch_auto_script_response::Event::LogMessage(log_message) => {
                                                    AutoScriptEvent::LogMessage{log_message}
                                                }
                                                crate::auto_drive_edge_service::v1::watch_auto_script_response::Event::ScriptExited(_) => {
                                                    AutoScriptEvent::Exited
                                                }
                                            };
                                            let mut history_lock = history.write().await;
                                            let _ = sender.send(event.clone());
                                            history_lock.push(event);
                                        }
                                    }
                                });
                            } else {
                                warn!("Could not watch script"); // TODO propagate this to user
                            }

                            let _ = response_tx.send(StartAutoScriptResponse::Uuid {
                                uuid: run_uuid.clone(),
                            });
                        } else {
                            let _ =
                                response_tx.send(StartAutoScriptResponse::EdgeClientUnavailable);
                        }
                    }
                    ToEdgeMessage::CancelAutoScript { run_uuid } => {
                        let _ = client
                            .cancel_auto_script(CancelAutoScriptRequest { run_uuid })
                            .await;
                    }
                    ToEdgeMessage::WatchAutoScript {
                        run_uuid,
                        response_tx,
                    } => {
                        let log_sender_data = {
                            let map = the_beginner_car_auto_script_events.lock().unwrap();
                            map.get(&run_uuid)
                                .map(|sender| (sender.history.clone(), sender.live.clone()))
                        };

                        if let Some((history, live)) = log_sender_data {
                            let history_lock = history.read().await;
                            let history_snapshot = history_lock.clone();
                            let mut grpc_receiver = live.subscribe();
                            drop(history_lock);
                            let (domain_sender, domain_receiver) =
                                mpsc::channel::<AutoScriptEvent>(100);

                            tokio::spawn(async move {
                                for event in history_snapshot {
                                    if domain_sender.send(event).await.is_err() {
                                        return; // Stop if the receiver dropped
                                    }
                                }

                                while let Ok(grpc_event) = grpc_receiver.recv().await {
                                    let domain_event = match grpc_event {
                                        AutoScriptEvent::LogMessage { log_message } => {
                                            AutoScriptEvent::LogMessage { log_message }
                                        }
                                        AutoScriptEvent::Exited => AutoScriptEvent::Exited,
                                    };

                                    // Stop forwarding if all receivers drop
                                    if domain_sender.send(domain_event).await.is_err() {
                                        break;
                                    }
                                }
                            });

                            let _ = response_tx.send(domain_receiver);
                        }
                    }
                    ToEdgeMessage::WatchOverview { response_tx } => {
                        let latest_auto_scripts_overview =
                            latest_auto_scripts_overview.lock().unwrap().clone();

                        let _ = response_tx.send((
                            latest_auto_scripts_overview,
                            incoming_auto_scripts_tx.subscribe(),
                        ));
                    }
                    ToEdgeMessage::WatchHardware { response_tx } => {
                        let _ = response_tx.send((
                            hardware_states_tx.borrow().clone(),
                            hardware_states_tx.subscribe(),
                        ));
                    }
                };
            }
        }
    });

    Ok(Hardware {
        logs: incoming_tx,
        sender,
        latest_auto_scripts_overview,
        watch_auto_scripts_overview: incoming_auto_scripts_tx,
        the_beginner_car_auto_script_events,
        hardware_states_tx,
    })
}
