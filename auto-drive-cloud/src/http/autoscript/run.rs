use std::convert::Infallible;
use std::time::Duration;

use futures_util::{StreamExt, stream};
use tokio::sync::oneshot;
use warp::{http::StatusCode, Filter, Reply, Rejection};
use bytes::Buf;
use crate::edge_client::{Hardware, ToDeviceMessage};
use crate::auto_drive_edge_service::v1::run_auto_script_response;

pub fn run(
    hardware: Hardware,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let logs = hardware.logs.clone();
    let sender = hardware.sender.clone();

    const MAX_BODY_SIZE: u64 = 16 * 1024;

    warp::path!("autoscript" / "run")
        .and(warp::post())
        // Parse multipart form data instead of raw bytes, applying the size limit here
        .and(warp::multipart::form().max_length(MAX_BODY_SIZE))
        .and_then(move |mut form: warp::multipart::FormData| {
            let logs = logs.clone();
            let sender = sender.clone();
            async move {
                let mut wasm_data = None;
                let mut args_string = String::new();

                // Extract fields from the multipart form
                while let Some(Ok(mut part)) = form.next().await {
                    match part.name() {
                        "file" => {
                            let mut data = Vec::new();
                            while let Some(Ok(chunk)) = part.data().await {
                                // Use .chunk() to get the &[u8] slice
                                data.extend_from_slice(chunk.chunk());
                            }
                            wasm_data = Some(data);
                        }
                        "args" => {
                            let mut data = Vec::new();
                            while let Some(Ok(chunk)) = part.data().await {
                                // Use .chunk() to get the &[u8] slice
                                data.extend_from_slice(chunk.chunk());
                            }
                            args_string = String::from_utf8_lossy(&data).to_string();
                        }
                        _ => {}
                    }
                }

                // Verify file was provided
                let wasm_bytes = match wasm_data {
                    Some(b) => b,
                    None => {
                        return Ok::<_, Infallible>(
                            warp::reply::with_status(
                                "Missing 'file' in multipart form",
                                StatusCode::BAD_REQUEST,
                            )
                            .into_response(),
                        );
                    }
                };

                // Validate arguments length limit (< 50 chars)
                if args_string.chars().count() >= 50 {
                    return Ok::<_, Infallible>(
                        warp::reply::with_status(
                            "Arguments string must be less than 50 characters",
                            StatusCode::BAD_REQUEST,
                        )
                        .into_response(),
                    );
                }

                let receiver = logs.subscribe();
                let (sender_start, receiver_start) = oneshot::channel();
                
                // Add `args` to your StartAutoScript struct/enum variant
                let message = ToDeviceMessage::StartAutoScript {
                    data: wasm_bytes,
                    uploader_comment: args_string,
                    response_tx: sender_start,
                };

                if sender.send(message).await.is_err() {
                    return Ok::<_, Infallible>(
                        warp::reply::with_status(
                            "Device channel is closed",
                            StatusCode::SERVICE_UNAVAILABLE,
                        )
                        .into_response(),
                    );
                }

                let streamed_response = receiver_start.await.unwrap().unwrap();

                let event_stream = stream::once(async {
                    Ok::<_, Infallible>(
                        warp::sse::Event::default()
                            .event("log")
                            .data("[INFO] [AUTO_DRIVE_CLOUD] Got the run WASM request and sent to the edge"),
                    )
                });

                let event_stream = event_stream.chain(
                    streamed_response.filter_map(|result| async move {
                        let result = result.unwrap();

                        match result.event {
                            Some(run_auto_script_response::Event::RunUuid(run_uuid)) => {
                                let run_uuid = serde_json::json!({ "run_uuid": run_uuid }).to_string();
                                Some(Ok::<_, Infallible>(
                                    warp::sse::Event::default()
                                        .event("log")
                                        .data(run_uuid),
                                ))
                            }
                            Some(run_auto_script_response::Event::Log(log)) => {
                                let log = log.log?;
                                let level = match log.level() {
                                    crate::auto_drive_edge_service::v1::LogLevel::Unspecified => "UNSPECIFIED",
                                    crate::auto_drive_edge_service::v1::LogLevel::Error => "ERROR",
                                    crate::auto_drive_edge_service::v1::LogLevel::Warn => "WARN",
                                    crate::auto_drive_edge_service::v1::LogLevel::Info => "INFO",
                                    crate::auto_drive_edge_service::v1::LogLevel::Debug => "DEBUG",
                                    crate::auto_drive_edge_service::v1::LogLevel::Trace => "TRACE",
                                };

                                let system_origin = match log.system_origin() {
                                    crate::auto_drive_edge_service::v1::LogSystemOrigin::WasmUnspecified => "UNSPECIFIED",
                                    crate::auto_drive_edge_service::v1::LogSystemOrigin::WasmMcu => "WASM_MCU",
                                    crate::auto_drive_edge_service::v1::LogSystemOrigin::WifiMcu => "WIFI_MCU",
                                };
                                
                                let formatted_log = format!("[{}] [{}] {}", level, system_origin, log.message);
                                Some(Ok::<_, Infallible>(
                                    warp::sse::Event::default()
                                        .event("log")
                                        .data(formatted_log),
                                ))
                            },
                            None => {
                                Some(Ok(
                                    warp::sse::Event::default()
                                        .event("error")
                                        .data("error.to_string()"),
                                ))
                            }
                        }
                    })
                );

                Ok(warp::sse::reply(
                    warp::sse::keep_alive()
                        .interval(Duration::from_secs(15))
                        .text("keep-alive")
                        .stream(event_stream),
                )
                .into_response())
            }
        })
        .recover(move |err: Rejection| async move {
            // Note: warp::multipart::form().max_length() payload errors will likely manifest as a warp::reject::PayloadTooLarge
            if err.find::<warp::reject::PayloadTooLarge>().is_some() {
                const MAX_BODY_KB: u64 = MAX_BODY_SIZE / 1024;
                let error_msg = format!("Payload too large. The maximum allowed size is {} KB.", MAX_BODY_KB);
                Ok(warp::reply::with_status(error_msg, StatusCode::PAYLOAD_TOO_LARGE).into_response())
            } else {
                Err(err)
            }
        })
}
