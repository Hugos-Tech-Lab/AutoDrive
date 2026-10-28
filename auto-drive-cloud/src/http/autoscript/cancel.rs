use bytes::Bytes;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use std::convert::Infallible;
use warp::{Filter, Reply, http::StatusCode};

use crate::edge_client::{Hardware, ToEdgeMessage};

#[derive(Debug, Deserialize)]
pub struct CancelRequest {
    pub run_uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String, // User ID in Supabase
    exp: usize,  // Expiration timestamp
    aud: String, // Audience ("authenticated")
}

fn validate_supabase_jwt_offline(token: &str, jwt_secret: &str) -> bool {
    let key = DecodingKey::from_secret(jwt_secret.as_bytes());

    // Supabase Auth tokens use HS256 by default
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&["authenticated"]);

    decode::<Claims>(token, &key, &validation).is_ok()
}

async fn validate_supabase_jwt(token: &str) -> Result<bool, reqwest::Error> {
    let supabase_url = std::env::var("SUPABASE_URL").unwrap_or_default();
    let anon_key = std::env::var("SUPABASE_ANON_KEY").unwrap_or_default();

    let client = reqwest::Client::new();
    let url = format!("{}/auth/v1/user", supabase_url);

    let response = client
        .get(url)
        .header("apikey", &anon_key)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    // HTTP 200 OK means the token is valid and active
    Ok(response.status().is_success())
}

pub fn cancel(
    hardware: Hardware,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let sender = hardware.sender.clone();

    warp::path!("autoscript" / "cancel")
        .and(warp::post())
        // Accept optional/empty JSON bodies gracefully
        .and(warp::body::bytes())
        .and_then(move |bytes: Bytes| {
            let sender = sender.clone();
            async move {
                // Parse run_uuid if present in JSON payload
                let cancel_req: CancelRequest = if bytes.is_empty() {
                    CancelRequest { run_uuid: None }
                } else {
                    serde_json::from_slice(&bytes).unwrap_or(CancelRequest { run_uuid: None })
                };

                let message = ToEdgeMessage::CancelAutoScript {
                    run_uuid: cancel_req.run_uuid.unwrap(),
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

                Ok::<_, Infallible>(
                    warp::reply::with_status("Cancel request sent", StatusCode::OK).into_response(),
                )
            }
        })
}
