use bytes::Bytes;
use serde::Deserialize;
use std::convert::Infallible;
use warp::{Filter, Reply, http::StatusCode};

use crate::edge_client::EdgeClient;

#[derive(Debug, Deserialize)]
pub struct CancelRequest {
    pub run_uuid: String,
}

pub fn cancel(
    edge: EdgeClient,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("autoscript" / "cancel")
        .and(warp::post())
        .and(warp::body::bytes())
        .and_then(move |bytes: Bytes| {
            let edge = edge.clone();
            async move {
                let cancel_request = match serde_json::from_slice::<CancelRequest>(&bytes) {
                    Ok(cancel_request) => cancel_request,
                    Err(_) => {
                        return Ok::<_, Infallible>(
                            warp::reply::with_status(
                                "Could not deserialize request",
                                StatusCode::PRECONDITION_FAILED,
                            )
                            .into_response(),
                        );
                    }
                };

                let cancel_response = edge.scripts.cancel(cancel_request.run_uuid).await;
                match cancel_response {
                    Ok(_response) => {}
                    Err(err) => {
                        return Ok::<_, Infallible>(
                            warp::reply::with_status(
                                format!("Could not cancel with error ({})", err),
                                StatusCode::PRECONDITION_FAILED,
                            )
                            .into_response(),
                        );
                    }
                }

                Ok::<_, Infallible>(
                    warp::reply::with_status("Cancel request sent", StatusCode::OK).into_response(),
                )
            }
        })
}
