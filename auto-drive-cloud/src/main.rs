use std::error::Error;

use tokio::sync::broadcast;
use tokio_stream::StreamExt;

use crate::{auto_drive_edge_service::v1::{WatchTheBeginnerCarEventsRequest, auto_drive_edge_service_client::AutoDriveEdgeServiceClient, watch_the_beginner_car_events_response::Event}, http::http_server};

pub mod hardware;
pub mod http;
pub mod edge_client;

pub mod auto_drive_edge_service {
    pub mod v1 {
        tonic::include_proto!("auto_drive_edge_service.v1");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let edge_client_ = edge_client::edge_client().await?;
    http_server(edge_client_).await;

    Ok(())
}
