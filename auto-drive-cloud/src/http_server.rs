use log::info;
use warp::Filter;

use crate::{
    edge_client::EdgeClient,
    http_server::{
        auto_script::{
            cancel::auto_script_cancel, enqueue::auto_script_enqueue, logs::auto_script_logs,
            overview::auto_script_overview, run::auto_script_run,
        },
        vehicle::overview::vehicle_overview,
    },
};
pub mod auto_script;
pub mod vehicle;

pub async fn http_server(edge: EdgeClient) {
    let server = tokio::spawn(async move {
        let auto_script_cancel = auto_script_cancel(edge.clone());
        let auto_script_enqueue = auto_script_enqueue(edge.clone());
        let auto_script_logs = auto_script_logs(edge.clone());
        let auto_script_overview = auto_script_overview(edge.clone());
        let auto_script_run = auto_script_run(edge.clone());

        let vehicle_overview = vehicle_overview(edge.clone());

        let cors = warp::cors()
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allow_headers(vec!["content-type", "authorization"]);

        #[cfg(feature = "development")]
        let cors = cors.allow_origin("http://localhost:5173");
        #[cfg(feature = "production")]
        let cors = cors.allow_origin("https://autodrive.hugostechlab.com");

        let routes = auto_script_cancel
            .or(auto_script_enqueue)
            .or(auto_script_logs)
            .or(auto_script_overview)
            .or(auto_script_run)
            .or(vehicle_overview)
            .with(cors);

        let address = ([0, 0, 0, 0], 3030);
        info!("serving HTTP to {address:?}");

        warp::serve(routes).run(address).await;
    });

    server.await.unwrap();
}
