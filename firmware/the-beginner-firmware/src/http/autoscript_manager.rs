use std::{sync::Arc, time::Duration};

use esp_idf_svc::{
    hal::reset,
    http::{Method, server::EspHttpServer},
};

use serde::Serialize;

use crate::autoscript::AutoScript;

pub fn set_handles(server: &mut EspHttpServer, auto_script: Arc<AutoScript>) -> anyhow::Result<()> {
    server.fn_handler(
        "/autoscript/upload",
        Method::Options,
        |req| -> anyhow::Result<()> {
            req.into_response(
                204,
                None,
                &[
                    ("Access-Control-Allow-Origin", "*"),
                    ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                    ("Access-Control-Allow-Headers", "Content-Type"),
                ],
            )?;

            Ok(())
        },
    )?;

    server.fn_handler(
        "/autoscript/upload",
        Method::Post,
        move |mut req| -> anyhow::Result<()> {
            let content_length = req
                .header("Content-Length")
                .and_then(|v| v.parse::<usize>().ok());

            log::info!(
                "Starting autoscript update, content length: {:?}",
                content_length
            );

            let mut data = Vec::with_capacity(content_length.unwrap_or(4096));

            let mut buf = [0u8; 4096];
            let mut total = 0usize;

            loop {
                let read = req.read(&mut buf)?;

                if read == 0 {
                    break;
                }

                data.extend_from_slice(&buf[..read]);
                total += read;

                log::debug!("Autoscript: received {} bytes", total);
            }

            log::info!("Autoscript download complete: {} bytes", total);

            auto_script.install(data).unwrap(); // TODO: remove unwrap

            let mut response = req.into_response(
                200,
                None,
                &[
                    ("Content-Type", "application/json"),
                    ("Access-Control-Allow-Origin", "*"),
                ],
            )?;

            #[derive(Serialize)]
            struct FirmwareUpdate200Response {
                status: String,
            }

            let body = serde_json::to_vec(&FirmwareUpdate200Response {
                status: "ok".to_string(),
            })?;
            response.write(&body)?;

            Ok(())
        },
    )?;

    Ok(())
}
