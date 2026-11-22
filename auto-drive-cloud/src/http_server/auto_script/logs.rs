use std::convert::Infallible;
use std::time::Duration;

use crate::{auto_drive_edge_service::format_log, edge_client::EdgeClient};
use futures_util::StreamExt;
use serde::Deserialize;
use warp::{Filter, Reply, http::StatusCode};

#[derive(Debug, Deserialize)]
pub struct AutoScriptLogsRequest {
    pub run_uuid: String,
}

pub fn autoscript_logs(
    hardware: EdgeClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("autoscript" / "logs")
        .and(warp::get())
        .and(warp::body::json::<AutoScriptLogsRequest>())
        .and_then(move |request: AutoScriptLogsRequest| {
            let edge = hardware.clone();

            async move {
                let auto_script = edge
                    .scripts
                    .find_auto_script(request.run_uuid, Duration::from_secs(1))
                    .await;

                let auto_script = match auto_script {
                    Ok(auto_script) => auto_script,
                    Err(err) => {
                        return Ok::<_, warp::Rejection>(
                            warp::reply::with_status(
                                format!("Failed to find auto script ({err})"),
                                StatusCode::BAD_REQUEST,
                            )
                            .into_response(),
                        );
                    }
                };

                let logs_stream = auto_script.logs.stream_with_history().await;

                let logs_stream = logs_stream.filter_map(|auto_script_event| async move {
                    let log = auto_script_event.unwrap();
                    let formatted_log = format_log(&log);
                    Some(Ok::<_, Infallible>(
                        warp::sse::Event::default().event("log").data(formatted_log),
                    ))
                });

                Ok(warp::sse::reply(
                    warp::sse::keep_alive()
                        .interval(Duration::from_secs(15))
                        .text("keep-alive")
                        .stream(logs_stream),
                )
                .into_response())
            }
        })
}
