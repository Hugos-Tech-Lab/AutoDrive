use crate::{auto_drive_edge_service::v1::RunAutoScriptRequest, edge_client::EdgeClient};
use serde::{Deserialize, Serialize};
use warp::{Filter, http::StatusCode};

#[derive(Deserialize)]
pub struct RunAutoScriptPayload {
    pub run_uuid: String,
    pub number_plate: String,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

pub fn autoscript_run(
    edge_client: EdgeClient,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let edge_client = edge_client.handle.clone();

    warp::path!("autoscript" / "run")
        .and(warp::post())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::body::json::<RunAutoScriptPayload>())
        .and_then(
            move |_auth_header: Option<String>, payload: RunAutoScriptPayload| {
                let mut edge_client = edge_client.clone();

                async move {
                    let request = RunAutoScriptRequest {
                        run_uuid: payload.run_uuid,
                        number_plate: payload.number_plate,
                    };

                    match edge_client.run_auto_script(request).await {
                        Ok(response) => {
                            let reply = warp::reply::json(&ApiResponse {
                                success: true,
                                message: format!("Script sent to hardware: {:?}", response),
                            });
                            Ok::<_, warp::Rejection>(warp::reply::with_status(
                                reply,
                                StatusCode::OK,
                            ))
                        }
                        Err(err) => {
                            let reply = warp::reply::json(&ApiResponse {
                                success: false,
                                message: format!("Failed to run auto script: {}", err),
                            });
                            Ok::<_, warp::Rejection>(warp::reply::with_status(
                                reply,
                                StatusCode::INTERNAL_SERVER_ERROR,
                            ))
                        }
                    }
                }
            },
        )
}
