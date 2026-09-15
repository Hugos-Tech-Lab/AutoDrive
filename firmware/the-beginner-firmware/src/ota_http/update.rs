use std::sync::{Arc, Mutex};
use std::time::Duration;

use esp_idf_svc::{
    hal::reset,
    http::{server::EspHttpServer, Method},
    ota::EspOta,
};

use serde::Serialize;

#[derive(Serialize)]
struct FirmwareUpdate200Response {
    status: String,
    message: String
}

pub fn set_handles(
    server: &mut EspHttpServer,
    ota: Arc<Mutex<EspOta>>,
) -> anyhow::Result<()> {
    server.fn_handler(
        "/firmware/update",
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
        "/firmware/update",
        Method::Post,
        move |mut req| -> anyhow::Result<()> {
            let content_length = req
                .header("Content-Length")
                .and_then(|v| v.parse::<usize>().ok());

            log::info!(
                "Starting OTA update, content length: {:?}",
                content_length
            );

            let mut ota = ota
                .lock()
                .map_err(|_| anyhow::anyhow!("OTA mutex poisoned"))?;

            let mut update = match content_length {
                Some(size) => ota.initiate_update_with_known_size(size)?,
                None => ota.initiate_update()?,
            };

            let mut buf = [0u8; 4096];
            let mut total = 0usize;

            loop {
                let read = req.read(&mut buf)?;

                if read == 0 {
                    break;
                }

                update.write(&buf[..read])?;
                total += read;

                log::debug!("OTA: received {} bytes", total);
            }

            log::info!("OTA upload complete: {} bytes", total);

            update.complete()?;

            let mut response = req.into_response(
                200,
                None,
                &[
                    ("Content-Type", "application/json"),
                    ("Access-Control-Allow-Origin", "*"),
                ],
            )?;


            let body = serde_json::to_vec(&FirmwareUpdate200Response{ status: "ok".to_string(), message: "Firmware installed, rebooting".to_string() })?;
            response.write(&body)?;

            log::info!("Rebooting...");

            std::thread::spawn(|| {
                std::thread::sleep(Duration::from_secs(1));
                log::info!("Rebooting...");
                reset::restart();
            });

            Ok(())
        },
    )?;

    server.fn_handler(
        "/firmware/update",
        Method::Get,
        move |req| -> anyhow::Result<()> {
            let device = update()?;

            let body = serde_json::to_vec(&device)?;

            let mut response = req.into_response(
                200,
                None,
                &[
                    ("Content-Type", "application/json"),
                    ("Access-Control-Allow-Origin", "*"),
                ],
            )?;

            response.write(&body)?;

            Ok(())
        },
    )?;

    Ok(())
}

fn update() -> anyhow::Result<()> {
    Ok(())
}