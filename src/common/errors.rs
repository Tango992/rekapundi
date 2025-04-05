use axum::{Json, extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse};
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
    /// Error related to ValidationErrors. This is different from the JsonRejection.
    ValidationError(validator::ValidationErrors),
    /// Error related to SQLx database operations.
    SqlxError(sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),

            AppError::ValidationError(errors) => {
                (StatusCode::UNPROCESSABLE_ENTITY, errors.to_string())
            }

            AppError::SqlxError(error) => match error {
                sqlx::Error::Database(db_error) => match db_error.kind() {
                    UniqueViolation | ForeignKeyViolation | NotNullViolation => {
                        (StatusCode::CONFLICT, db_error.to_string())
                    }
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, db_error.to_string()),
                },

                sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, error.to_string()),

                _ => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            },
        };

        match status {
            StatusCode::INTERNAL_SERVER_ERROR => {
                tracing::error!("Internal server error: {}", message);
                status.into_response()
            }
            _ => (status, Json(ErrorResponse { message })).into_response(),
        }
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::JsonRejection(rejection)
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
