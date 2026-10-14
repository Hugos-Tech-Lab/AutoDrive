use std::convert::Infallible;
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::BroadcastStream;
use warp::Filter;

use crate::auto_drive_edge_service::v1::watch_the_beginner_car_events_response::Event;

#[derive(Serialize, Deserialize)]
pub enum VisualizeEvent {
    IrSensorReading { value: u32 },
}

pub fn hardware_telemetry(
    sender: Sender<Event>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("hardware_telemetry").and(warp::get()).and_then(move || {
        let receiver = sender.subscribe();

        async move {
            let event_stream = BroadcastStream::new(receiver).filter_map(|result| async move {
                match result {
                    Ok(Event::IrSensorAReading(reading)) => {
                        // let event = VisualizeEvent::IrSensorReading { value: reading.value };

                        Some(Ok::<_, Infallible>(
                            warp::sse::Event::default().event("ir_sensor_a_reading").data(format!("{}", reading.value)),
                            // warp::sse::Event::default().event("log").data(serde_json::to_string(&event).unwrap()),
                        ))
                    }
                    Ok(_) => None,
                    Err(error) => {
                        eprintln!("Log broadcast receiver error: {error}");

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
