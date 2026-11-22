use std::convert::Infallible;
use std::time::Duration;

use crate::edge_client::EdgeClient;
use futures_util::StreamExt;
use tokio_stream::wrappers::WatchStream;
use warp::{Filter, Reply};

pub fn autoscript_overview(
    hardware: EdgeClient,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let scripts_overview = hardware.scripts_overview.clone();

    warp::path!("autoscript" / "overview")
        .and(warp::get())
        .and_then(move || {
            let rx = scripts_overview.subscribe();

            async move {
                let event_stream = WatchStream::new(rx).map(|state| {
                    Ok::<_, Infallible>(
                        warp::sse::Event::default()
                            .event("overview")
                            .json_data(&state)
                            .unwrap_or_else(|_| warp::sse::Event::default().data("{}")),
                    )
                });

                Ok::<_, warp::Rejection>(
                    warp::sse::reply(
                        warp::sse::keep_alive()
                            .interval(Duration::from_secs(15))
                            .text("keep-alive")
                            .stream(event_stream),
                    )
                    .into_response(),
                )
            }
        })
}
