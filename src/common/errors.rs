use axum::{
    Json,
    extract::rejection::{JsonRejection, PathRejection},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use sqlx::error::ErrorKind::{ForeignKeyViolation, NotNullViolation, UniqueViolation};

/// Custom error type for API responses.
#[derive(Serialize)]
struct ErrorResponse {
    /// Error message.
    message: String,
}

/// Enum representing different types of application errors.
/// It implements the `IntoResponse` trait to convert errors into HTTP responses.
pub enum AppError {
    /// Error related to Axum's JSON extraction.
    JsonRejection(JsonRejection),
    /// Error related to Axum's Path extraction.
    PathRejection(PathRejection),
    /// Error related to SQLx database operations.
    SqlxError(sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::JsonRejection(rejection) => (rejection.status(), Some(rejection.body_text())),

            AppError::PathRejection(rejection) => (
                rejection.status(),
                Some("Invalid path parameter".to_string()),
            ),

            AppError::SqlxError(error) => match error {
                sqlx::Error::Database(db_error) => match db_error.kind() {
                    UniqueViolation | ForeignKeyViolation | NotNullViolation => {
                        tracing::debug!("{:?}", db_error.to_string());
                        (StatusCode::CONFLICT, None)
                    }

                    _ => {
                        tracing::debug!("{:?}", db_error.to_string());
                        (StatusCode::INTERNAL_SERVER_ERROR, None)
                    }
                },

                sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, None),

                _ => {
                    tracing::debug!("{:?}", error.to_string());
                    (StatusCode::INTERNAL_SERVER_ERROR, None)
                }
            },
        };

        match message {
            Some(msg) => (status, Json(ErrorResponse { message: msg })).into_response(),
            None => status.into_response(),
        }
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::JsonRejection(rejection)
    }
}

impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> Self {
        AppError::PathRejection(rejection)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::SqlxError(error)
    }
}

/// Only tests easily testable code without mocking or external dependencies.
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use serde::{Deserialize, Serialize};

    // Test struct for validation errors
    #[derive(Debug, Serialize, Deserialize)]
    struct TestUser {
        username: String,
    }

    #[test]
    fn test_from_sqlx_row_not_found_error() {
        let sqlx_error = sqlx::Error::RowNotFound;
        let app_error = AppError::from(sqlx_error);

        assert!(matches!(app_error, AppError::SqlxError(_)));
        assert_eq!(app_error.into_response().status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_from_sqlx_unhandled_error() {
        let sqlx_error = sqlx::Error::Protocol("Test error".to_string());
        let app_error = AppError::from(sqlx_error);

        assert!(matches!(app_error, AppError::SqlxError(_)));
        assert_eq!(
            app_error.into_response().status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
