use std::convert::Infallible;
use serde::Deserialize;
use bytes::Bytes;
use warp::{http::StatusCode, Filter, Reply, Rejection};

use crate::edge_client::{Hardware, ToEdgeMessage};

#[derive(Debug, Deserialize)]
pub struct CancelRequest {
    pub run_uuid: Option<String>,
}

pub fn cancel(
    hardware: Hardware,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let sender = hardware.sender.clone();

    warp::path!("autoscript" / "cancel")
        .and(warp::post())
        // Accept optional/empty JSON bodies gracefully
        .and(warp::body::bytes())
        .and_then(move |bytes: Bytes| {
            let sender = sender.clone();
            async move {
                // Parse run_uuid if present in JSON payload
                let cancel_req: CancelRequest = if bytes.is_empty() {
                    CancelRequest { run_uuid: None }
                } else {
                    serde_json::from_slice(&bytes).unwrap_or(CancelRequest { run_uuid: None })
                };

                let message = ToEdgeMessage::CancelAutoScript {
                    run_uuid: cancel_req.run_uuid.unwrap(),
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

                Ok::<_, Infallible>(
                    warp::reply::with_status(
                        "Cancel request sent",
                        StatusCode::OK,
                    )
                    .into_response(),
                )
            }
        })
}
