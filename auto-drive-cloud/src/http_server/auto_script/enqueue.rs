use std::convert::Infallible;
use std::time::Duration;

use crate::{auto_drive_edge_service::format_log, edge_client::EdgeClient};
use bytes::Buf;
use futures_util::{StreamExt, stream};
use warp::{Filter, Rejection, Reply, http::StatusCode};

pub fn auto_script_enqueue(
    edge_client: EdgeClient,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    const MAX_BODY_SIZE: u64 = 16 * 1024;

    warp::path!("autoscript" / "enqueue")
        .and(warp::post())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::multipart::form().max_length(MAX_BODY_SIZE))
        .and_then(
            move |auth_header: Option<String>, form: warp::multipart::FormData| {
                let edge_client = edge_client.clone();
                async move { handle_enqueue(auth_header, form, edge_client).await }
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

async fn handle_enqueue(
    auth_header: Option<String>,
    mut form: warp::multipart::FormData,
    edge_client: EdgeClient,
) -> Result<warp::reply::Response, Infallible> {
    let expected_token = "your_secure_token_here".to_string();
    let owner_name = "joe".to_string(); // TODO: get the owner name from token somehow?

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

    let mut wasm_data = None;
    let mut owner_comment = String::new();

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
                owner_comment = String::from_utf8_lossy(&data).to_string();
            }
            _ => {}
        }
    }

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

    if owner_comment.chars().count() >= 50 {
        return Ok(warp::reply::with_status(
            "Comment must be less than 50 characters",
            StatusCode::BAD_REQUEST,
        )
        .into_response());
    }

    let run_uuid = edge_client
        .scripts
        .enqueue(owner_name, owner_comment, wasm_bytes, 0)
        .await;

    let run_uuid = match run_uuid {
        Ok(run_uuid) => run_uuid,
        Err(err) => {
            return Ok(warp::reply::with_status(
                format!("Error ({err}) when enqueue"),
                StatusCode::BAD_REQUEST,
            )
            .into_response());
        }
    };

    let auto_script = edge_client
        .scripts
        .find_auto_script(run_uuid.clone(), Duration::from_secs(5))
        .await;

    let auto_script = match auto_script {
        Ok(auto_script) => auto_script,
        Err(err) => {
            return Ok(warp::reply::with_status(
                format!("Failed to find auto script ({err})"),
                StatusCode::BAD_REQUEST,
            )
            .into_response());
        }
    };

    let logs = auto_script.logs.stream_with_history().await;

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
        let run_uuid = serde_json::json!({ "run_uuid": run_uuid.clone() }).to_string();
        Ok::<_, Infallible>(warp::sse::Event::default().event("log").data(run_uuid))
    }));

    let event_stream = run_uuid_stream.chain(logs.filter_map(|auto_script_event| async move {
        let log = auto_script_event.unwrap();
        let formatted_log = format_log(&log.into());
        Some(Ok::<_, Infallible>(
            warp::sse::Event::default().event("log").data(formatted_log),
        ))
    }));

    let event_stream = event_stream.chain(exit);

    Ok(warp::sse::reply(
        warp::sse::keep_alive()
            .interval(Duration::from_secs(15))
            .text("keep-alive")
            .stream(event_stream),
    )
    .into_response())
}
