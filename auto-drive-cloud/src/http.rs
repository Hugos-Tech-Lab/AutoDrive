use tokio::sync::broadcast::Sender;
use warp::Filter;

use crate::{auto_drive_edge_service::v1::watch_the_beginner_car_events_response::Event, edge_client::Hardware, http::{autoscript::run::run, logs::logs}};
use crate::http::hardware_telemetry::hardware_telemetry;
pub mod logs;
pub mod autoscript;
pub mod hardware_telemetry;

pub async fn http_server(receiver: Hardware) {
    let server = tokio::spawn(async move {
        let watch_logs = logs(receiver.logs.clone());
        let run = run(receiver.clone());
        let hardware_telemetry = hardware_telemetry(receiver.logs);
        
        let address = ([127, 0, 0, 1], 3030);
        println!("serving http to {address:?}");
        warp::serve(watch_logs.or(run).or(hardware_telemetry)).run(address).await;
    });

    server.await.unwrap();
}