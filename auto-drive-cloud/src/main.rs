use std::error::Error;

use env_logger::Env;
use log::info;

use crate::{
    edge_client::EdgeClient, http::http_server, ignore_incomplete_message::IgnoreIncompleteMessage,
};
pub mod edge_client;
pub mod hardware;
pub mod http;
pub mod ignore_incomplete_message;

pub mod auto_drive_edge_service {
    pub mod v1 {
        tonic::include_proto!("auto_drive_edge_service.v1");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let logger = env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_source_path(true)
        .build(); // Enables file and line number

    let max_level = logger.filter();
    log::set_boxed_logger(Box::new(IgnoreIncompleteMessage(logger))).unwrap();
    log::set_max_level(max_level);

    #[cfg(feature = "development")]
    info!("auto-drive-cloud launched in development mode");

    #[cfg(feature = "production")]
    {
        const GIT_HASH: &str = env!("GIT_HASH");
        info!("Starting auto-drive-edge in production mode at git hash ({GIT_HASH})");
    }

    let edge_client_ = EdgeClient::new().await?;
    http_server(edge_client_).await;

    Ok(())
}

#[cfg(all(feature = "development", feature = "production"))]
compile_error!("Choose either development or production");

#[cfg(not(any(feature = "development", feature = "production")))]
compile_error!("Choose a build environment");
