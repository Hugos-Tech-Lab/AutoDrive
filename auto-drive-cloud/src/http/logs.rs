use std::convert::Infallible;
use std::time::Duration;

use futures_util::StreamExt;
use log::warn;
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::BroadcastStream;
use warp::Filter;

use crate::auto_drive_edge_service::v1::watch_the_beginner_car_events_response::Event;

pub fn logs(
    sender: Sender<Event>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("logs").and(warp::get()).and_then(move || {
        let receiver = sender.subscribe();

        async move {
            let event_stream = BroadcastStream::new(receiver).filter_map(|result| async move {
                match result {
                    Ok(Event::Log(log_event)) => {
                        let log = log_event.log?;

                        Some(Ok::<_, Infallible>(
                            warp::sse::Event::default().event("log").data(log.message),
                        ))
                    }
                    Ok(_) => None,
                    Err(error) => {
                        warn!("Log broadcast receiver error: {error}");

                        Some(Ok::<_, Infallible>(
                            warp::sse::Event::default()
                                .event("error")
                                .data(error.to_string()),
                        ))
                    }
                }
            });

            Ok::<_, warp::Rejection>(warp::sse::reply(
                warp::sse::keep_alive()
                    .interval(Duration::from_secs(15))
                    .text("keep-alive")
                    .stream(event_stream),
            ))
        }
    })
}
