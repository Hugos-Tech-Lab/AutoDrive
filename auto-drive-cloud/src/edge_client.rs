use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tokio::sync::{
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
    WatchHardware {
        response_tx: oneshot::Sender<(HardwareHashMap, watch::Receiver<HardwareHashMap>)>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDeviceState {
    pub id: String,
    pub category: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone)]
pub struct Hardware {
    pub sender: mpsc::Sender<ToEdgeMessage>,
    pub logs: Sender<Event>,
    pub the_beginner_car_auto_scripts: Arc<Mutex<HashMap<String, Sender<LogMessage>>>>,
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
                println!("Successfully connected to edge service.");
                break c;
            }
            Err(err) => {
                eprintln!("Failed to connect to edge service: {err}. Retrying in 5s...");
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
    let the_beginner_car_auto_scripts = Arc::new(Mutex::new(HashMap::new()));

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
                                    eprintln!(
                                        "Beginner car event stream disconnected: {err}. Reconnecting..."
                                    );
                                    break;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!(
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
                                    eprintln!(
                                        "Auto script overview stream disconnected: {err}. Reconnecting..."
                                    );
                                    break;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!(
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
                                            let entry = v.entry(hardware.id.clone()).or_insert(
                                                HardwareDeviceState {
                                                    id: hardware.id,
                                                    category: hardware.category,
                                                    is_connected: hardware.is_connected,
                                                },
                                            );
                                            entry.is_connected = hardware.is_connected;
                                        }
                                    });
                                }
                                Err(err) => {
                                    eprintln!(
                                        "Hardware status stream disconnected: {err}. Reconnecting..."
                                    );
                                    break;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!(
                            "Failed to subscribe to hardware status: {err}. Retrying in 5s..."
                        );
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
        let the_beginner_car_auto_scripts = the_beginner_car_auto_scripts.clone();
        let hardware_states_tx = hardware_states_tx.clone();
        async move {
            while let Some(value) = receiver.recv().await {
                match value {
                    ToEdgeMessage::StartAutoScript {
                        owner,
                        data,
                        uploader_comment: _,
                        response_tx,
                    } => {
                        dbg!("starting");
                        if let Ok(response) = client
                            .run_auto_script(RunAutoScriptRequest {
                                owner,
                                build: 1,
                                data,
                            })
                            .await
                        {
                            let response = response.into_inner();
                            let run_uuid = response.run_uuid;

                            let (sender, _) = broadcast::channel(100);

                            the_beginner_car_auto_scripts
                                .lock()
                                .unwrap()
                                .insert(run_uuid.clone(), sender.clone());

                            if let Ok(watch_auto_script) = client
                                .watch_auto_script(WatchAutoScriptRequest {
                                    run_uuid: run_uuid.clone(),
                                })
                                .await
                            {
                                let mut stream = watch_auto_script.into_inner();

                                tokio::spawn(async move {
                                    while let Ok(Some(update)) = stream.message().await {
                                        if let Some(log_message) = update.log_message {
                                            let _ = sender.send(log_message);
                                        }
                                    }
                                });
                            }
                            let _ = response_tx.send(run_uuid.clone());
                        } else {
                            eprintln!(
                                "Failed to start auto script - service might be temporarily unavailable."
                            );
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
                        if let Some(log_sender) =
                            the_beginner_car_auto_scripts.lock().unwrap().get(&run_uuid)
                        {
                            let _ = response_tx.send(log_sender.subscribe());
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
        the_beginner_car_auto_scripts,
        hardware_states_tx,
    })
}
