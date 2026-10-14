use std::error::Error;

use tokio::sync::{broadcast::{self, Sender}, mpsc};
use tokio_stream::StreamExt;

use crate::auto_drive_edge_service::v1::{RunAutoScriptRequest, WatchTheBeginnerCarEventsRequest, auto_drive_edge_service_client::AutoDriveEdgeServiceClient, watch_the_beginner_car_events_response::Event};

pub enum ToDeviceMessage {
  StartAutoScript { data: Vec<u8> }
}

pub struct Hardware {
  pub logs: Sender<Event>,
  pub sender: mpsc::Sender<ToDeviceMessage>
}

pub async fn edge_client() -> Result<Hardware, Box<dyn Error>> {
    let mut client = AutoDriveEdgeServiceClient::connect("http://127.0.0.1:50051").await?;

    let (incoming_tx, _) = broadcast::channel::<Event>(100);

    tokio::spawn({
        let incoming_tx = incoming_tx.clone();
        let mut client = client.clone();
        async move {
            let response = client
                .watch_the_beginner_car_events(WatchTheBeginnerCarEventsRequest{})
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

    let (sender, mut receiver) = mpsc::channel::<ToDeviceMessage>(10);
    tokio::spawn(async move {
      while let Some(value) = receiver.recv().await {
        match value {
            ToDeviceMessage::StartAutoScript { data } => client.run_auto_script(RunAutoScriptRequest{ build: 1, data }).await.unwrap(),
        };
      }
    });

    Ok(Hardware {
        logs: incoming_tx,
        sender,
    })
}