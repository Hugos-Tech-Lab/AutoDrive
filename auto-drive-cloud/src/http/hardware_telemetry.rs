use std::convert::Infallible;
use std::time::Duration;

use futures_util::StreamExt;
use log::warn;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::BroadcastStream;
use warp::Filter;

use crate::auto_drive_edge_service::v1::watch_the_beginner_car_events_response::Event;

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct InfraredLedReading {
    front_far_left: u32,
    front_left: u32,
    front_center: u32,
    front_right: u32,
    front_far_right: u32,
    back_left: u32,
    back_center: u32,
}

pub fn hardware_telemetry(
    sender: Sender<Event>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("hardware_telemetry")
        .and(warp::get())
        .and_then(move || {
            let receiver = sender.subscribe();

            async move {
                let event_stream = BroadcastStream::new(receiver).filter_map(|result| async move {
                    match result {
                        Ok(Event::InfraredLedReading(infrared_led_reading)) => {
                            let infrared_led_reading = InfraredLedReading {
                                front_far_left: infrared_led_reading.front_far_left,
                                front_left: infrared_led_reading.front_left,
                                front_center: infrared_led_reading.front_center,
                                front_right: infrared_led_reading.front_right,
                                front_far_right: infrared_led_reading.front_far_right,
                                back_left: infrared_led_reading.back_left,
                                back_center: infrared_led_reading.back_center,
                            };

                            Some(Ok::<_, Infallible>(
                                warp::sse::Event::default()
                                    .event("infrared_led_reading")
                                    .data(serde_json::to_string(&infrared_led_reading).unwrap()),
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
