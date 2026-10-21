use std::convert::Infallible;
use std::time::Duration;

use crate::edge_client::{Hardware, ToEdgeMessage};
use futures_util::{StreamExt, stream};
use tokio::sync::oneshot;
use tokio_stream::wrappers::WatchStream;
use warp::{Filter, Reply, http::StatusCode};

pub fn hardware_overview(
    hardware: Hardware,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let sender = hardware.sender.clone();

    warp::path!("hardware" / "overview")
        .and(warp::get())
        .and_then(move || {
            let sender = sender.clone();
            async move {
                let (response_tx, response_rx) = oneshot::channel();

                let message = ToEdgeMessage::WatchHardware { response_tx };

                if sender.send(message).await.is_err() {
                    return Ok::<_, Infallible>(
                        warp::reply::with_status(
                            "Device channel is closed",
                            StatusCode::SERVICE_UNAVAILABLE,
                        )
                        .into_response(),
                    );
                }

                let (initial_state, receiver) = match response_rx.await {
                    Ok(res) => res,
                    Err(_) => {
                        return Ok::<_, Infallible>(
                            warp::reply::with_status(
                                "Failed to get response from edge",
                                StatusCode::INTERNAL_SERVER_ERROR,
                            )
                            .into_response(),
                        );
                    }
                };

                let initial_stream = stream::once(async move {
                    Ok::<_, Infallible>(
                        warp::sse::Event::default()
                            .event("overview")
                            .json_data(&initial_state)
                            .unwrap_or_else(|_| warp::sse::Event::default().data("{}")),
                    )
                });

                let update_stream = WatchStream::new(receiver).filter_map(|result| async move {
                    match result {
                        state => Some(Ok::<_, Infallible>(
                            warp::sse::Event::default()
                                .event("overview")
                                .json_data(&state)
                                .unwrap_or_else(|_| warp::sse::Event::default().data("{}")),
                        ))
                    }
                });

                let event_stream = initial_stream.chain(update_stream);

                Ok(warp::sse::reply(
                    warp::sse::keep_alive()
                        .interval(Duration::from_secs(15))
                        .text("keep-alive")
                        .stream(event_stream),
                )
                .into_response())
            }
        })
}
