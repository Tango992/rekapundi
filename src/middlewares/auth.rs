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
    static ref JWT_SECRET: String = env::var("JWT_SECRET")
        .inspect_err(|_| {
            tracing::error!("JWT_SECRET environment variable is not set");
        })
        .unwrap();
}

/// The payload of the JWT token.
#[derive(Debug, Deserialize, Serialize)]
struct Claim {
    /// The expiration time of the token in unix timestamp.
    exp: usize,
    /// The unix timestamp before which the token is not valid.
    nbf: usize,
}

/// Middleware to authenticate requests using JWT tokens.
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
        middleware::from_fn,
        routing::get,
    };
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serial_test::serial;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::ServiceExt;

    const TEST_JWT_SECRET: &str = "fufufafa";

    /// Helper to create a Claim for testing
    fn create_test_claim(exp_offset_secs: i64, nbf_offset_secs: i64) -> Claim {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Claim {
            exp: (now as i64 + exp_offset_secs) as usize,
            nbf: (now as i64 + nbf_offset_secs) as usize,
        }
    }

    /// Helper to generate a JWT token for testing
    fn generate_test_token(claim: &Claim, secret: &str) -> String {
        encode(
            &Header::default(),
            claim,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap()
    }

    // Setup the router with the middleware
    fn setup_test_router() -> Router {
        unsafe { std::env::set_var("JWT_SECRET", TEST_JWT_SECRET) };

        Router::new()
            .route("/test", get(|| async { StatusCode::OK }))
            .route_layer(from_fn(authenticate_request))
    }

    #[tokio::test]
    #[serial]
    async fn test_authenticate_request_valid_token() {
        // Prepare
        let app = setup_test_router();
        let claim = create_test_claim(3600, -60); // Expires in 1 hour, valid 1 minute ago
        let token = generate_test_token(&claim, TEST_JWT_SECRET);

        let request = Request::builder()
            .uri("/test")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[serial]
    async fn test_authenticate_request_no_auth_header() {
        // Prepare
        let app = setup_test_router();

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_authenticate_request_invalid_header_format() {
        // Prepare
        let app = setup_test_router();

        let request = Request::builder()
            .uri("/test")
            .header(header::AUTHORIZATION, "InvalidFormat")
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_authenticate_request_invalid_token_signature() {
        // Prepare
        let app = setup_test_router();
        let claim = create_test_claim(3600, -60);
        let token = generate_test_token(&claim, "wrong_secret");

        let request = Request::builder()
            .uri("/test")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_authenticate_request_expired_token() {
        // Prepare
        let app = setup_test_router();
        let claim = create_test_claim(-3600, -3660); // Expired 1 hour ago, valid 1 hour + 1 min ago
        let token = generate_test_token(&claim, TEST_JWT_SECRET);

        let request = Request::builder()
            .uri("/test")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_authenticate_request_not_yet_valid_token() {
        // Prepare
        let app = setup_test_router();
        let claim = create_test_claim(3600, 60); // Expires in 1 hour, valid in 1 minute
        let token = generate_test_token(&claim, TEST_JWT_SECRET);

        let request = Request::builder()
            .uri("/test")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_authenticate_request_missing_exp_claim() {
        // Prepare
        let app = setup_test_router();
        #[derive(Debug, Deserialize, Serialize)]
        struct ClaimMissingExp {
            nbf: usize,
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claim = ClaimMissingExp {
            nbf: now as usize - 60, // Valid 1 min ago
        };

        let token = encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(TEST_JWT_SECRET.as_ref()),
        )
        .unwrap();

        let request = Request::builder()
            .uri("/test")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_authenticate_request_missing_nbf_claim() {
        // Prepare
        let app = setup_test_router();

        #[derive(Debug, Deserialize, Serialize)]
        struct ClaimMissingNbf {
            exp: usize,
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claim = ClaimMissingNbf {
            exp: now as usize + 3600, // Expires in 1 hour
        };

        let token = encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(TEST_JWT_SECRET.as_ref()),
        )
        .unwrap();

        let request = Request::builder()
            .uri("/test")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
