use std::{
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use esp_idf_svc::http::{Method, server::EspHttpServer};

use serde::Serialize;

use crate::autoscript::{AutoScript, AutoScriptRunProgress};

pub fn set_handles(server: &mut EspHttpServer, auto_script: Arc<AutoScript>) -> anyhow::Result<()> {
    // --- Existing CORS OPTIONS Handler ---
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

    // --- Existing POST Upload Handler ---
    server.fn_handler("/autoscript/upload", Method::Post, {
        let auto_script = auto_script.clone();
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

            auto_script
                .install(data)
                .map_err(|e| anyhow::anyhow!("Failed to install script: {:?}", e))?;

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
        }
    })?;

    server.fn_handler(
        "/autoscript/run",
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

    // --- New /autoscript/run Streaming Handler ---
    let auto_script_run = auto_script.clone();
    server.fn_handler(
        "/autoscript/run",
        Method::Post,
        move |req| -> anyhow::Result<()> {
            log::info!("Starting autoscript execution stream...");
            // 1. Create a channel to stream logs/chunks from the background task back to HTTP handler
            let (tx, rx) = mpsc::sync_channel::<AutoScriptRunProgress>(10);

            let auto_script_clone = auto_script_run.clone();

            thread::Builder::new()
                .name("autoscript_exec".into())
                .stack_size(8192)
                .spawn(move || {
                    auto_script_clone.run(tx).unwrap();
                })
                .unwrap();

            // 3. Prepare chunked streaming HTTP response (specify chunked Transfer-Encoding)
            let mut response = req
                .into_response(
                    200,
                    None,
                    &[
                        ("Content-Type", "application/x-ndjson"), // or text/event-stream / text/plain
                        ("Transfer-Encoding", "chunked"),
                        ("Access-Control-Allow-Origin", "*"),
                    ],
                )
                .unwrap();

            // 4. Stream data chunks directly to client as they are produced
            while let Ok(chunk) = rx.recv() {
                let bytes = serde_json::to_vec(&chunk)?;

                response.write(&bytes).unwrap();
                // Flush explicitly to send the chunk over TCP immediately
                response.flush().unwrap();
            }

            log::info!("Autoscript execution stream ended.");
            Ok(())
        },
    )?;

    Ok(())
}
