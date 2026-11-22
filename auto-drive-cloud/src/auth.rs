use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
enum Role {
    User,
    Admin,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    aud: String,
    role: Role,
}

fn authenticate_jwt(token: &str, jwt_secret: &str) -> bool {
    let key = DecodingKey::from_secret(jwt_secret.as_bytes());

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&["authenticated"]);

    decode::<Claims>(token, &key, &validation).is_ok()
}

// async fn validate_supabase_jwt(token: &str) -> Result<bool, reqwest::Error> {
//     let supabase_url = std::env::var("SUPABASE_URL").unwrap_or_default();
//     let anon_key = std::env::var("SUPABASE_ANON_KEY").unwrap_or_default();

//     let client = reqwest::Client::new();
//     let url = format!("{}/auth/v1/user", supabase_url);

//     let response = client
//         .get(url)
//         .header("apikey", &anon_key)
//         .header("Authorization", format!("Bearer {}", token))
//         .send()
//         .await?;

//     // HTTP 200 OK means the token is valid and active
//     Ok(response.status().is_success())
// }
