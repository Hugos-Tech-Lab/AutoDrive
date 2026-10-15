use std::convert::Infallible;
use std::time::Duration;

use bytes::Bytes;
use futures_util::{StreamExt, stream};
use tokio::sync::oneshot;
use tokio_stream::wrappers::BroadcastStream;
use warp::{http::StatusCode, Filter, Reply, Rejection}; // Added Rejection

use crate::auto_drive_edge_service::v1::watch_the_beginner_car_events_response::Event;
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
        .and(warp::body::content_length_limit(MAX_BODY_SIZE))
        .and(warp::body::bytes())
        .and_then(move |body: Bytes| {
            let logs = logs.clone();
            let sender = sender.clone();
            async move {
                let receiver = logs.subscribe();

                let (sender_start, receiver_start) = oneshot::channel();
                let message = ToDeviceMessage::StartAutoScript {
                    data: body.to_vec(),
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


                // let event_stream = event_stream.chain(stream::once(async move {
                //     Ok::<_, Infallible>(
                //         warp::sse::Event::default()
                //             .event("log")
                //             .data(serde_json::json!({ "run_uuid": run_uuid }).to_string())
                //     )
                // }));

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
                                let level = log.level();
                                let level = match level {
                                    crate::auto_drive_edge_service::v1::LogLevel::Unspecified => "UNSPECIFIED", // TODO: cause actual error instead
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
                                // eprintln!("Log broadcast receiver error");

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
            if err.find::<warp::reject::PayloadTooLarge>().is_some() {
                const MAX_BODY_KB: u64 = MAX_BODY_SIZE / 1024; // Calculate KB for the error message
                let error_msg = format!("Payload too large. The maximum allowed size is {} KB.", MAX_BODY_KB);
                Ok(warp::reply::with_status(error_msg, StatusCode::PAYLOAD_TOO_LARGE).into_response())
            } else {
                // If it's a different error, pass it along
                Err(err)
            }
        })
}