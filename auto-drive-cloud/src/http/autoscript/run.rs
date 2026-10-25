use std::convert::Infallible;
use std::time::Duration;

use crate::edge_client::{AutoScriptEvent, Hardware, StartAutoScriptResponse, ToEdgeMessage};
use bytes::Buf;
use futures_util::{StreamExt, stream};
use tokio::sync::oneshot;
use tokio_stream::wrappers::ReceiverStream;
use warp::{Filter, Rejection, Reply, http::StatusCode};

pub fn run(
    hardware: Hardware,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let sender = hardware.sender.clone();

    const MAX_BODY_SIZE: u64 = 16 * 1024;

    warp::path!("autoscript" / "run")
        .and(warp::post())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::multipart::form().max_length(MAX_BODY_SIZE))
        .and_then(
            move |auth_header: Option<String>, form: warp::multipart::FormData| {
                let sender = sender.clone();
                async move { handle_autoscript_run(auth_header, form, sender).await }
            },
        )
        .recover(move |err: Rejection| async move {
            if err.find::<warp::reject::PayloadTooLarge>().is_some() {
                const MAX_BODY_KB: u64 = MAX_BODY_SIZE / 1024;
                let error_msg = format!(
                    "Payload too large. The maximum allowed size is {} KB.",
                    MAX_BODY_KB
                );
                Ok(
                    warp::reply::with_status(error_msg, StatusCode::PAYLOAD_TOO_LARGE)
                        .into_response(),
                )
            } else {
                Err(err)
            }
        })
}

async fn handle_autoscript_run(
    auth_header: Option<String>,
    mut form: warp::multipart::FormData,
    sender: tokio::sync::mpsc::Sender<ToEdgeMessage>,
) -> Result<warp::reply::Response, Infallible> {
    // --- AUTHENTICATION CHECK ---
    let expected_token = "your_secure_token_here".to_string();

    let is_authorized = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let provided_token = header.trim_start_matches("Bearer ");
            provided_token == expected_token && !expected_token.is_empty()
        }
        _ => false,
    };

    if !is_authorized {
        return Ok(warp::reply::with_status(
            "Unauthorized: Invalid or missing token",
            StatusCode::UNAUTHORIZED,
        )
        .into_response());
    }
    // ----------------------------

    let mut wasm_data = None;
    let mut args_string = String::new();

    // Extract fields from the multipart form
    while let Some(Ok(mut part)) = form.next().await {
        match part.name() {
            "file" => {
                let mut data = Vec::new();
                while let Some(Ok(chunk)) = part.data().await {
                    data.extend_from_slice(chunk.chunk());
                }
                wasm_data = Some(data);
            }
            "args" => {
                let mut data = Vec::new();
                while let Some(Ok(chunk)) = part.data().await {
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
            return Ok(warp::reply::with_status(
                "Missing 'file' in multipart form",
                StatusCode::BAD_REQUEST,
            )
            .into_response());
        }
    };

    // Validate arguments length limit (< 50 chars)
    if args_string.chars().count() >= 50 {
        return Ok(warp::reply::with_status(
            "Arguments string must be less than 50 characters",
            StatusCode::BAD_REQUEST,
        )
        .into_response());
    }

    let (sender_start, receiver_start) = oneshot::channel();

    let message = ToEdgeMessage::StartAutoScript {
        data: wasm_bytes,
        uploader_comment: args_string,
        response_tx: sender_start,
        owner: "hugoooo".to_string(),
    };

    if sender.send(message).await.is_err() {
        return Ok(warp::reply::with_status(
            "Device channel is closed",
            StatusCode::SERVICE_UNAVAILABLE,
        )
        .into_response());
    }

    let start_auto_script_response = receiver_start.await.unwrap();
    let started_script_uuid = match start_auto_script_response {
        StartAutoScriptResponse::Uuid { uuid } => uuid,
        StartAutoScriptResponse::EdgeClientUnavailable => {
            return Ok(warp::reply::with_status(
                "Edge client (the PC that runs in the lab) is unavailable",
                StatusCode::SERVICE_UNAVAILABLE,
            )
            .into_response());
        }
    };

    let (sender_start, receiver_start) = oneshot::channel();

    let message = ToEdgeMessage::WatchAutoScript {
        run_uuid: started_script_uuid.clone(),
        response_tx: sender_start,
    };
    if sender.send(message).await.is_err() {
        return Ok(warp::reply::with_status(
            "Device channel is closed",
            StatusCode::SERVICE_UNAVAILABLE,
        )
        .into_response());
    }

    let log_stream = receiver_start.await;
    let streamed_response = ReceiverStream::new(log_stream.unwrap());

    let confirmation = stream::once(async {
        Ok::<_, Infallible>(warp::sse::Event::default().event("log").data(
            "[INFO] [AUTO_DRIVE_CLOUD] Got the run WASM request and sent to the edge (PC in lab)",
        ))
    });

    let exit = stream::once(async {
        Ok::<_, Infallible>(
            warp::sse::Event::default()
                .event("log")
                .data("[INFO] [AUTO_DRIVE_CLOUD] Exiting. Bye."),
        )
    });

    let run_uuid_stream = confirmation.chain(stream::once(async move {
        let run_uuid = serde_json::json!({ "run_uuid": started_script_uuid }).to_string();
        Ok::<_, Infallible>(warp::sse::Event::default().event("log").data(run_uuid))
    }));

    let event_stream = run_uuid_stream.chain(
        streamed_response
            .take_while(|auto_script_event| {
                let keep = !matches!(auto_script_event, AutoScriptEvent::Exited);
                std::future::ready(keep)
            })
            .filter_map(|auto_script_event| async move {
                let log = match auto_script_event {
                    AutoScriptEvent::LogMessage { log_message } => log_message,
                    AutoScriptEvent::Exited => {
                        return None;
                    }
                };

                let level = match log.level() {
                    crate::auto_drive_edge_service::v1::LogLevel::Unspecified => "UNSPECIFIED",
                    crate::auto_drive_edge_service::v1::LogLevel::Error => "ERROR",
                    crate::auto_drive_edge_service::v1::LogLevel::Warn => "WARN",
                    crate::auto_drive_edge_service::v1::LogLevel::Info => "INFO",
                    crate::auto_drive_edge_service::v1::LogLevel::Debug => "DEBUG",
                    crate::auto_drive_edge_service::v1::LogLevel::Trace => "TRACE",
                };

                let system_origin = match log.system_origin() {
                    crate::auto_drive_edge_service::v1::LogSystemOrigin::WasmUnspecified => {
                        "UNSPECIFIED"
                    }
                    crate::auto_drive_edge_service::v1::LogSystemOrigin::WasmMcu => "WASM_MCU",
                    crate::auto_drive_edge_service::v1::LogSystemOrigin::WifiMcu => "WIFI_MCU",
                };

                let formatted_log = format!("[{}] [{}] {}", level, system_origin, log.message);
                Some(Ok::<_, Infallible>(
                    warp::sse::Event::default().event("log").data(formatted_log),
                ))
            }),
    );

    let event_stream = event_stream.chain(exit);

    Ok(warp::sse::reply(
        warp::sse::keep_alive()
            .interval(Duration::from_secs(15))
            .text("keep-alive")
            .stream(event_stream),
    )
    .into_response())
}
