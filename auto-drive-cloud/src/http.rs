use tokio::sync::broadcast::Sender;
use warp::Filter;

use crate::{auto_drive_edge_service::v1::watch_the_beginner_car_events_response::Event, edge_client::Hardware, http::{autoscript::{cancel::cancel, run::run, watch::autoscript_watch}, logs::logs}};
use crate::http::hardware_telemetry::hardware_telemetry;
pub mod logs;
pub mod autoscript;
pub mod hardware_telemetry;

pub async fn http_server(receiver: Hardware) {
    let server = tokio::spawn(async move {
        let watch_logs = logs(receiver.logs.clone());
        let run = run(receiver.clone());
        let cancel = cancel(receiver.clone());
        let autoscript_watch = autoscript_watch(receiver.clone());
        let hardware_telemetry = hardware_telemetry(receiver.logs);

        let cors = warp::cors()
            .allow_origin("http://localhost:5173")
            .allow_methods(vec!["GET", "POST", "OPTIONS"])
            .allow_headers(vec!["content-type"]);

        let routes = watch_logs
            .or(run)
            .or(hardware_telemetry)
            .or(cancel)
            .or(autoscript_watch)
            .with(cors);

        let address = ([127, 0, 0, 1], 3030);
        println!("serving HTTP to {address:?}");

        warp::serve(routes).run(address).await;
    });

    server.await.unwrap();
}