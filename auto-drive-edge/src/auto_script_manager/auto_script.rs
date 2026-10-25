use std::sync::Arc;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use the_beginner_car_tcp_protocol::{
    LogLevel, LogMessage, LogSystemOrigin, TheBeginnerCarOutgoingMessages,
};
use tokio::{
    sync::{
        RwLock,
        broadcast::{self, Sender},
    },
    task::JoinHandle,
};
use uuid::Uuid;

use crate::{
    auto_script_manager::{AutoScriptEvents, AutoScriptResponseFromHardware},
    the_beginner_car_comms::TheBeginnerCarComms,
};

#[derive(Clone, Debug)]
pub struct AutoScript {
    pub owner_name: String,
    pub run_uuid: String,
    pub run_number: u32,
    pub uploaded_script: Arc<[u8]>,
    pub uploaded_script_hash: String,
    pub long_lived_hardware_listener_handle: Arc<JoinHandle<()>>,
    pub events_tx: Sender<AutoScriptEvents>,
    pub events_history: Arc<RwLock<Vec<AutoScriptEvents>>>,
    pub hardware: TheBeginnerCarComms,
    pub hardware_response: AutoScriptResponseFromHardware,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
}

impl AutoScript {
    pub fn new(
        owner_name: String,
        run_number: u32,
        uploaded_script: Arc<[u8]>,
        hardware: TheBeginnerCarComms,
    ) -> Self {
        let run_uuid = Uuid::new_v4().to_string();

        let uploaded_script_hash = Sha256::digest(&uploaded_script);
        let uploaded_script_hash = hex::encode(Sha256::digest(uploaded_script_hash));

        let (events_tx, _) = broadcast::channel(100);
        let events_history = Arc::new(RwLock::new(Vec::new()));

        let long_lived_hardware_listener_handle = tokio::task::spawn({
            let hardware_rx = hardware.incoming_tx.subscribe();
            let run_uuid = run_uuid.clone();
            let events_history = events_history.clone();
            let events_tx = events_tx.clone();

            async move {
                AutoScript::long_lived_hardware_listener(
                    hardware_rx,
                    events_history,
                    events_tx,
                    run_uuid,
                )
                .await
            }
        });

        Self {
            owner_name,
            run_uuid,
            run_number,
            uploaded_script,
            uploaded_script_hash,
            long_lived_hardware_listener_handle: Arc::from(long_lived_hardware_listener_handle),
            events_tx,
            events_history,
            hardware,
            hardware_response: AutoScriptResponseFromHardware::HeardNothing,
            queued_at: Utc::now(),
            started_at: None,
            stopped_at: None,
        }
    }

    pub async fn long_lived_hardware_listener(
        mut hardware_rx: broadcast::Receiver<TheBeginnerCarOutgoingMessages>,
        events_history: Arc<RwLock<Vec<AutoScriptEvents>>>,
        events_tx: broadcast::Sender<AutoScriptEvents>,
        run_uuid: String,
    ) {
        loop {
            // TODO: add `tokio::select!` here later to handle the cancellation token
            match hardware_rx.recv().await {
                Ok(message) => match message {
                    TheBeginnerCarOutgoingMessages::Log { log } => {
                        if log.trace_id.as_deref() == Some(&run_uuid) {
                            let event = AutoScriptEvents::LogMessage {
                                log_message: log.clone(),
                            };
                            let mut hist_lock = events_history.write().await;
                            hist_lock.push(event.clone());
                            let _ = events_tx.send(event);
                        }
                    }
                    TheBeginnerCarOutgoingMessages::AutoScriptStopped => {
                        let event = AutoScriptEvents::ScriptExited;
                        let mut hist_lock = events_history.write().await;
                        hist_lock.push(event.clone());
                        let _ = events_tx.send(event);
                    }
                    _ => {}
                },
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    eprintln!("Car watcher lagged; skipped {skipped} messages");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    // The car disconnected or channel closed, exit the loop
                    break;
                }
            };
        }
    }

    pub async fn send_log_info(&self, message: String) {
        let event = AutoScriptEvents::LogMessage {
            log_message: LogMessage {
                system_origin: LogSystemOrigin::WasmMcu, // TODO: change this origin to Edge.
                level: LogLevel::Info,                   // TODO: support for other than info.
                target: "".to_string(),                  // TODO: what does target mean?.
                message: message,
                trace_id: Some(self.run_uuid.clone()),
            },
        };
        let mut hist_lock = self.events_history.write().await;
        hist_lock.push(event.clone());
        let _ = self.events_tx.send(event);
    }
}
