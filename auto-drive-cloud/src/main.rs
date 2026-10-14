use std::error::Error;

use tokio::sync::broadcast;
use tokio_stream::StreamExt;

use crate::{auto_drive_edge_service::v1::{WatchTheBeginnerCarEventsRequest, auto_drive_edge_service_client::AutoDriveEdgeServiceClient, watch_the_beginner_car_events_response::Event}, http::http_server};

pub mod hardware;
pub mod http;

pub mod auto_drive_edge_service {
    pub mod v1 {
        tonic::include_proto!("auto_drive_edge_service.v1");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = AutoDriveEdgeServiceClient::connect("http://127.0.0.1:50051").await?;

    let (incoming_tx, _) = broadcast::channel::<Event>(100);

    tokio::spawn({
        let incoming_tx = incoming_tx.clone();

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

    http_server(incoming_tx).await;


    Ok(())
}
