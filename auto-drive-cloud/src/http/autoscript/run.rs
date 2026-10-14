use std::convert::Infallible;
use std::time::Duration;

use bytes::Bytes;
use futures_util::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use warp::{http::StatusCode, Filter, Reply};

use crate::auto_drive_edge_service::v1::watch_the_beginner_car_events_response::Event;
use crate::edge_client::{Hardware, ToDeviceMessage};

pub fn run(
    hardware: Hardware,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let logs = hardware.logs.clone();
    let sender = hardware.sender.clone();

    warp::path!("autoscript" / "run")
        .and(warp::post())
        .and(warp::body::bytes())
        .and_then(move |body: Bytes| {
            let logs = logs.clone();
            let sender = sender.clone();

            async move {
                let receiver = logs.subscribe();

                let message = ToDeviceMessage::StartAutoScript {
                    data: body.to_vec(),
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

                let event_stream =
                    BroadcastStream::new(receiver).filter_map(|result| async move {
                        match result {
                            Ok(Event::Log(log_event)) => {
                                let log = log_event.log?;
                                let level = log.level();
                                let level = match level {
                                    crate::auto_drive_edge_service::v1::LogLevel::Unspecified => "UNSPECIFIED", // TODO: cause actual error instead
                                    crate::auto_drive_edge_service::v1::LogLevel::Error => "ERROR",
                                    crate::auto_drive_edge_service::v1::LogLevel::Warn => "WARN",
                                    crate::auto_drive_edge_service::v1::LogLevel::Info => "INFO",
                                    crate::auto_drive_edge_service::v1::LogLevel::Debug => "DEBUG",
                                    crate::auto_drive_edge_service::v1::LogLevel::Trace => "TRACE",
                                };
                                
                                let formatted_log = format!("[{}] {}", level, log.message);

                                Some(Ok::<_, Infallible>(
                                    warp::sse::Event::default()
                                        .event("log")
                                        .data(formatted_log),
                                ))
                            }
                            Ok(_) => None,
                            Err(error) => {
                                eprintln!("Log broadcast receiver error: {error}");

                                Some(Ok(
                                    warp::sse::Event::default()
                                        .event("error")
                                        .data(error.to_string()),
                                ))
                            }
                        }
                    });

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