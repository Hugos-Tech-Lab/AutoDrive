use log::info;
use warp::Filter;

use crate::http::hardware::overview::hardware_overview;
use crate::http::hardware_telemetry::hardware_telemetry;
use crate::{
    edge_client::Hardware,
    http::{
        autoscript::{
            cancel::cancel, logs::autoscript_watch, overview::autoscript_overview, run::run,
        },
        logs::logs,
    },
};
pub mod autoscript;
pub mod hardware;
pub mod hardware_telemetry;
pub mod logs;

pub async fn http_server(receiver: Hardware) {
    let server = tokio::spawn(async move {
        let watch_logs = logs(receiver.logs.clone());
        let run = run(receiver.clone());
        let cancel = cancel(receiver.clone());
        let autoscript_watch = autoscript_watch(receiver.clone());
        let autoscript_overview = autoscript_overview(receiver.clone());
        let hardware_overview = hardware_overview(receiver.clone());
        let hardware_telemetry = hardware_telemetry(receiver.logs);

        let cors = warp::cors()
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allow_headers(vec!["content-type", "authorization"]);

        #[cfg(feature = "development")]
        let cors = cors.allow_origin("http://localhost:5173");
        #[cfg(feature = "production")]
        let cors = cors.allow_origin("https://autodrive.hugostechlab.com");

        let routes = watch_logs
            .or(run)
            .or(hardware_telemetry)
            .or(cancel)
            .or(autoscript_watch)
            .or(autoscript_overview)
            .or(hardware_overview)
            .with(cors);

        let address = ([0, 0, 0, 0], 3030);
        info!("serving HTTP to {address:?}");

        warp::serve(routes).run(address).await;
    });

    server.await.unwrap();
}
