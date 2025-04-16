use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::IntoResponse,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref JWT_SECRET: String = env::var("JWT_SECRET").unwrap();
}

/// The payload of the JWT token.
#[derive(Debug, Deserialize, Serialize)]
struct Claim {
    /// The expiration time of the token in unix timestamp.
    exp: usize,
    /// The unix timestamp before which the token is not valid.
    nbf: usize,
}

pub async fn authenticate_request(
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let raw_token = auth_header
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .split("Bearer ")
        .nth(1)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let mut validation = Validation::default();
    validation.set_required_spec_claims(&["exp", "nbf"]);

    let token = decode::<Claim>(
        raw_token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &validation,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let current_unix_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| {
            tracing::error!("Failed to get current time");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .as_secs()
        .try_into()
        .map_err(|_| {
            tracing::error!("Failed to convert time to usize");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if token.claims.nbf > current_unix_timestamp {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}
