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
    /// Error related to ValidationErrors. This is different from the JsonRejection.
    ValidationError(validator::ValidationErrors),
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

            AppError::ValidationError(errors) => {
                (StatusCode::UNPROCESSABLE_ENTITY, Some(errors.to_string()))
            }

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

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        AppError::ValidationError(errors)
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
    use validator::Validate;

    // Test struct for validation errors
    #[derive(Debug, Validate, Serialize, Deserialize)]
    struct TestUser {
        #[validate(length(min = 3))]
        username: String,
    }

    #[test]
    fn test_from_validation_errors() {
        let test_user = TestUser {
            username: "ab".to_string(),
        };

        let validation_result = test_user.validate();

        let app_error = AppError::from(validation_result.err().unwrap());
        match app_error {
            AppError::ValidationError(_) => assert!(true),
            _ => panic!("Expected ValidationError variant"),
        }

        assert_eq!(
            app_error.into_response().status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
}
