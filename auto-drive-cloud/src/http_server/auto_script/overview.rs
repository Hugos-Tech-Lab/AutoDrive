use std::convert::Infallible;
use std::time::Duration;

use crate::edge_client::EdgeClient;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use warp::{Filter, Reply};

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
struct AutoScript {
    run_uuid: String,
    // TODO: more events here...
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct AutoScriptOverviewStreamedResponse {
    auto_scripts: Vec<AutoScript>,
}

pub fn auto_script_overview(
    edge: EdgeClient,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("autoscript" / "overview")
        .and(warp::get())
        .and_then(move || {
            let edge = edge.clone();
            async move {
                let scripts_overview = edge.scripts.stream();

                let event_stream = scripts_overview.map(|auto_scripts| {
                    let response = AutoScriptOverviewStreamedResponse {
                        auto_scripts: auto_scripts
                            .iter()
                            .map(|auto_script| AutoScript {
                                run_uuid: auto_script.run_uuid.clone(),
                            })
                            .collect(),
                    };

                    Ok::<_, Infallible>(
                        warp::sse::Event::default()
                            .event("overview")
                            .json_data(&response)
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
