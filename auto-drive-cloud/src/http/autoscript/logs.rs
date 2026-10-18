use std::convert::Infallible;
use std::time::Duration;

use crate::edge_client::Hardware;
use futures_util::StreamExt;
use serde::Serialize;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use warp::{Filter, Rejection, Reply};

// 1. Define the Rust structs mapping to your TypeScript types
#[derive(Serialize, Clone)]
pub struct TimeTrialDrivingEntry {
    pub name: String,
    pub run_time: String,
    pub upload_hash: String,
    pub run_number: u32,
    pub run_uuid: String,
    pub uploader_comment: String,
}

#[derive(Serialize, Clone)]
pub struct TimeTrialQueueEntry {
    pub name: String,
    pub waiting_time: String,
    pub upload_hash: String,
    pub run_number: u32,
    pub run_uuid: String,
    pub uploader_comment: String,
}

#[derive(Serialize, Clone)]
pub struct TimeTrialData {
    pub driving_now: Vec<TimeTrialDrivingEntry>,
    pub queue: Vec<TimeTrialQueueEntry>,
}

// 2. The new GET endpoint for SSE streaming
pub fn autoscript_watch(
    hardware: Hardware, // Assuming you need to pass state to fetch real queue data
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // Note the new endpoint name
    warp::path!("autoscript" / "watch")
        .and(warp::get()) // SSE requires GET, not POST
        .and_then(move || {
            let hardware = hardware.clone();

            async move {
                // Set up a 10-second tick interval for updates
                let timer = interval(Duration::from_secs(10));
                let timer_stream = IntervalStream::new(timer);

                let event_stream = timer_stream.map(move |_| {
                    // TODO: Replace this mock data with actual state from `hardware`
                    let data = TimeTrialData {
                        driving_now: vec![TimeTrialDrivingEntry {
                            name: "Magnus".to_string(),
                            run_time: "5s".to_string(), // Frontend will handle incrementing this locally
                            upload_hash: "a1b2c3d4e5".to_string(),
                            run_number: 42,
                            run_uuid: "8f3a9b12-4c...".to_string(),
                            uploader_comment: "Fast run".to_string(),
                        }],
                        queue: vec![TimeTrialQueueEntry {
                            name: "Joe".to_string(),
                            waiting_time: "2s".to_string(),
                            upload_hash: "f9e8d7c6b5".to_string(),
                            run_number: 43,
                            run_uuid: "3d2e1f0a-9b...".to_string(),
                            uploader_comment: "Smooth curve test".to_string(),
                        }],
                    };

                    // Serialize to JSON
                    let json_string =
                        serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string());

                    // Map to warp::sse::Event matching the Svelte 'update' listener
                    Ok::<_, Infallible>(
                        warp::sse::Event::default()
                            .event("update")
                            .data(json_string),
                    )
                });

                // Reply with the continuous SSE stream
                Ok::<_, Infallible>(
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
