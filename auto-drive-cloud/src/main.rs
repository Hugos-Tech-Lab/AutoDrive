use std::error::Error;

use crate::http::http_server;

pub mod edge_client;
pub mod hardware;
pub mod http;

pub mod auto_drive_edge_service {
    pub mod v1 {
        tonic::include_proto!("auto_drive_edge_service.v1");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "development")]
    println!("auto-drive-cloud launched in development mode");

    #[cfg(feature = "production")]
    {
        const GIT_HASH: &str = env!("GIT_HASH");
        log::info!("Starting auto-drive-edge in production mode at git hash ({GIT_HASH})");
    }

    let edge_client_ = edge_client::edge_client().await?;
    http_server(edge_client_).await;

    Ok(())
}

#[cfg(all(feature = "development", feature = "production"))]
compile_error!("Choose either development or production");

#[cfg(not(any(feature = "development", feature = "production")))]
compile_error!("Choose a build environment");
