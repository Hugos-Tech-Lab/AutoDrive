use std::sync::Arc;

use edge_http::io::Error;
use embedded_io_async::{Read, Write};
use futures::{FutureExt, select};
use log::info;

use crate::{
    http_server::{CORS_HEADERS, FirmwareUpdate200Response}, auto_script::{AutoScript, AutoScriptRunProgress},
};

pub async fn upload<T, const N: usize>(
    conn: &mut edge_http::io::server::Connection<'_, T, N>,
    auto_script: Arc<AutoScript>,
) -> Result<(), Error<T::Error>>
where
    T: Read + Write,
{
    info!("processing upload");

    let request_headers = conn.headers().unwrap();

    // edge-http usually parses content_len for us
    let content_length = request_headers.headers.content_len();
    log::info!(
        "Starting autoscript update, content length: {:?}",
        content_length
    );

    let capacity = content_length.map(|c| c as usize).unwrap_or(2048);
    let mut data = Vec::with_capacity(capacity);
    let mut buf = [0u8; 2048];
    let mut total = 0usize;
    // TODO: add progress to response
    loop {
        let read = conn.read(&mut buf).await.unwrap();
        if read == 0 {
            break;
        }
        data.extend_from_slice(&buf[..read]);
        total += read;
        log::debug!("Autoscript: received {} bytes", total);

        // Break early if we've reached the expected content length to avoid hanging
        // on keep-alive connections
        if let Some(cl) = content_length {
            if total >= cl as usize {
                break;
            }
        }
    }

    log::info!("Autoscript download complete: {} bytes", total);

    let res = auto_script.install(data).await;

    if let Ok(res) = res {
        log::error!("{:?}", res);
    }

    // if let Err(e) = res {
    //     log::error!("Failed to install script: {:?}", e);
    //     conn.initiate_response(500, Some("Internal Server Error"), &cors_headers).await?;
    //     return Ok(());
    // }

    let body = serde_json::to_vec(&FirmwareUpdate200Response {
        status: "ok".to_string(),
    })
    .unwrap();

    let mut res_headers = CORS_HEADERS.to_vec();
    res_headers.push(("Content-Type", "application/json"));

    conn.initiate_response(200, Some("OK"), &res_headers)
        .await?;
    conn.write_all(&body).await?;
    Ok(())
}

pub async fn run<T, const N: usize>(
    conn: &mut edge_http::io::server::Connection<'_, T, N>,
    auto_script: Arc<AutoScript>,
) -> Result<(), Error<T::Error>>
where
    T: Read + Write,
{
    log::info!("Starting autoscript execution stream...");
    let (tx, rx) = flume::bounded::<AutoScriptRunProgress>(10);

    let auto_script_clone = auto_script.clone();

    let mut res_headers = CORS_HEADERS.to_vec();
    res_headers.push(("Content-Type", "application/x-ndjson"));
    res_headers.push(("Transfer-Encoding", "chunked"));
    conn.initiate_response(200, Some("OK"), &res_headers)
        .await?;
    let auto_fut = auto_script_clone.run(tx).fuse();
    futures::pin_mut!(auto_fut);
    loop {
        select! {
            auto = auto_fut => {
                log::error!("done");
                // TODO: stuff with auto

                // function done
                break;
            }
            progress = rx.recv_async().fuse() => {
                match progress {
                    Ok(chunk) => {
                        let mut bytes = serde_json::to_vec(&chunk).unwrap();
                        bytes.push(b'\n'); // CRITICAL for NDJSON framing
                        let res = conn.write_all(&bytes).await;
                        if let Err(err) = res {
                            log::error!("failed to write bytes: '{:?}'", err);
                        }
                    }
                    Err(err) => {
                        log::error!("{:?}", err);
                        break;
                    },
                }

            }
        }
    }

    log::info!("Autoscript execution stream ended.");
    Ok(())
}

pub async fn cancel<T, const N: usize>(
    conn: &mut edge_http::io::server::Connection<'_, T, N>,
    auto_script: Arc<AutoScript>,
) -> Result<(), Error<T::Error>>
where
    T: Read + Write,
{
    log::info!("Starting cancel stream...");

    auto_script.cancel();

    log::info!("done stream...");

    let body = serde_json::to_vec(&FirmwareUpdate200Response {
        status: "ok".to_string(),
    })
    .unwrap();

    let mut res_headers = CORS_HEADERS.to_vec();
    res_headers.push(("Content-Type", "application/json"));

    conn.initiate_response(200, Some("OK"), &res_headers)
        .await?;
    conn.write_all(&body).await?;
    Ok(())
}
