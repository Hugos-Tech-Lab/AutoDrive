use crate::edge_client::EdgeClient;
use serde::{Deserialize, Serialize};
use warp::{Filter, Reply};

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct VehicleOverviewResponse {
    number_plates: Vec<String>,
}

pub fn vehicle_overview(
    edge: EdgeClient,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let number_plates: Vec<String> = edge
        .vehicles
        .iter()
        .map(|vehicle| vehicle.number_plate.clone())
        .collect();

    warp::path!("vehicle" / "overview")
        .and(warp::get())
        .map(move || {
            let number_plates = number_plates.clone();
            warp::reply::json(&VehicleOverviewResponse { number_plates }).into_response()
        })
}
