use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use axum_extra::extract::WithRejection;
use std::sync::Arc;

use crate::{
    common::errors::AppError,
    dtos::expense::{IndexExpenseQuery, IndexExpenseResponse, SaveBatchExpense, SaveExpense},
    repositories::expense,
};

/// Handles the routes related to expenses operations.
pub fn expense_routes() -> Router<Arc<dyn expense::RepositoryOperation>> {
    Router::new().nest(
        "/expenses",
        Router::new()
            .route("/", get(index))
            .route("/", post(save_bulk))
            .route("/{id}", get(show))
            .route("/{id}", put(update))
            .route("/{id}", delete(destroy))
            .route("/latest", get(show_latest)),
    )
}

/// Handles the deletion of a specific expense by ID.
async fn destroy(
    WithRejection(Path(id), _): WithRejection<Path<u32>, AppError>,
    State(expense_repository): State<Arc<dyn expense::RepositoryOperation>>,
) -> Result<impl IntoResponse, AppError> {
    expense_repository.delete(id as i32).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handles showing the list of expenses.
async fn index(
    Query(query): Query<IndexExpenseQuery>,
    State(expense_repository): State<Arc<dyn expense::RepositoryOperation>>,
) -> Result<impl IntoResponse, AppError> {
    let expenses = expense_repository.find_all(&query).await?;

    Ok((StatusCode::OK, Json(IndexExpenseResponse { expenses })))
}

/// Handles the bulk save of expenses.
async fn save_bulk(
    State(expense_repository): State<Arc<dyn expense::RepositoryOperation>>,
    WithRejection(Json(body), _): WithRejection<Json<SaveBatchExpense>, AppError>,
) -> Result<impl IntoResponse, AppError> {
    expense_repository.insert_bulk(&body.expenses).await?;

    Ok(StatusCode::CREATED)
}

/// Handles the retrieval of a specific expense by ID.
async fn show(
    WithRejection(Path(id), _): WithRejection<Path<u32>, AppError>,
    State(expense_repository): State<Arc<dyn expense::RepositoryOperation>>,
) -> Result<impl IntoResponse, AppError> {
    let expense = expense_repository.find_one(id as i32).await?;

    Ok((StatusCode::OK, Json(expense)))
}

/// Handles the retrieval of the latest expense.
async fn show_latest(
    State(expense_repository): State<Arc<dyn expense::RepositoryOperation>>,
) -> Result<impl IntoResponse, AppError> {
    let latest_expense = expense_repository.find_latest().await?;

    Ok((StatusCode::OK, Json(latest_expense)))
}

/// Handles the update of a specific expense by ID.
async fn update(
    WithRejection(Path(id), _): WithRejection<Path<u32>, AppError>,
    State(expense_repository): State<Arc<dyn expense::RepositoryOperation>>,
    WithRejection(Json(body), _): WithRejection<Json<SaveExpense>, AppError>,
) -> Result<impl IntoResponse, AppError> {
    expense_repository.update(id as i32, &body).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dtos::{
        expense::{IndexExpenseQuery, SaveExpense},
        query_result::{IndexExpenseElement, ShowExpense, ShowLatestExpense, SimpleEntity, Tag},
    };

    use async_trait::async_trait;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json;
    use sqlx::Error as SqlxError;
    use std::sync::Arc;
    use tower::ServiceExt;

    pub struct MockExpenseRepository;

    impl MockExpenseRepository {
        pub fn new() -> Arc<Self> {
            Arc::new(Self)
        }
    }

    fn index_expense_response() -> IndexExpenseResponse {
        IndexExpenseResponse {
            expenses: vec![
                IndexExpenseElement {
                    id: 1,
                    amount: 1000,
                    date: "2025-04-01".to_string(),
                    description: Some("Test expense 1".to_string()),
                },
                IndexExpenseElement {
                    id: 2,
                    amount: 2000,
                    date: "2025-04-02".to_string(),
                    description: None,
                },
            ],
        }
    }

    fn show_latest_expense_response() -> ShowLatestExpense {
        ShowLatestExpense {
            id: 3,
            amount: 3000,
            date: "2025-04-03".to_string(),
            description: Some("Latest test expense".to_string()),
            priority: 0,
            category: sqlx::types::Json(SimpleEntity {
                id: 1,
                name: "Food".to_string(),
            }),
            wallet: sqlx::types::Json(SimpleEntity {
                id: 1,
                name: "Cash".to_string(),
            }),
            tags: sqlx::types::Json(vec![
                Tag {
                    id: 1,
                    name: "Essential".to_string(),
                    is_important: true,
                },
                Tag {
                    id: 2,
                    name: "Groceries".to_string(),
                    is_important: false,
                },
            ]),
        }
    }

    fn show_expense_response(id: i32) -> ShowExpense {
        ShowExpense {
            amount: 1000,
            date: "2025-04-01".to_string(),
            description: Some(format!("Test expense {}", id)),
            priority: 1,
            category: sqlx::types::Json(SimpleEntity {
                id: 1,
                name: "Food".to_string(),
            }),
            wallet: sqlx::types::Json(SimpleEntity {
                id: 1,
                name: "Cash".to_string(),
            }),
            tags: sqlx::types::Json(vec![Tag {
                id: 1,
                name: "Essential".to_string(),
                is_important: true,
            }]),
        }
    }

    #[async_trait]
    impl expense::RepositoryOperation for MockExpenseRepository {
        async fn delete(&self, _id: i32) -> Result<(), SqlxError> {
            Ok(())
        }

        async fn find_all(
            &self,
            _query: &IndexExpenseQuery,
        ) -> Result<Vec<IndexExpenseElement>, SqlxError> {
            Ok(index_expense_response().expenses)
        }

        async fn find_latest(&self) -> Result<ShowLatestExpense, SqlxError> {
            Ok(show_latest_expense_response())
        }

        async fn find_one(&self, id: i32) -> Result<ShowExpense, SqlxError> {
            Ok(show_expense_response(id))
        }

        async fn insert_bulk(&self, _expenses: &[SaveExpense]) -> Result<(), SqlxError> {
            Ok(())
        }

        async fn update(&self, _id: i32, _expense: &SaveExpense) -> Result<(), SqlxError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_destroy_handler() {
        // Prepare
        let repo = MockExpenseRepository::new();
        let app = expense_routes().with_state(repo);

        let request = Request::builder()
            .method("DELETE")
            .uri("/expenses/1")
            .body(axum::body::Body::empty())
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_index_handler() {
        // Prepare
        let repo = MockExpenseRepository::new();
        let app = expense_routes().with_state(repo);

        let request = Request::builder()
            .method("GET")
            .uri("/expenses")
            .body(axum::body::Body::empty())
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<IndexExpenseResponse>(&body_bytes).unwrap();

        assert_eq!(body, index_expense_response());
    }

    #[tokio::test]
    async fn test_save_bulk_handler() {
        // Prepare
        let repo = MockExpenseRepository::new();
        let app = expense_routes().with_state(repo);

        // Use serde_json::json! macro to avoid serialization issues
        let request = Request::builder()
            .method("POST")
            .uri("/expenses")
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "expenses": [{
                        "amount": 1000,
                        "date": "2025-04-01",
                        "description": "Test expense",
                        "priority": 1,
                        "categoryId": 1,
                        "walletId": 1,
                        "tagIds": [1, 2]
                    }]
                })
                .to_string(),
            ))
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_show_handler() {
        // Prepare
        let repo = MockExpenseRepository::new();
        let app = expense_routes().with_state(repo);

        let request = Request::builder()
            .method("GET")
            .uri("/expenses/1")
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<ShowExpense>(&body_bytes).unwrap();

        assert_eq!(body, show_expense_response(1));
    }

    #[tokio::test]
    async fn test_show_latest_handler() {
        // Prepare
        let repo = MockExpenseRepository::new();
        let app = expense_routes().with_state(repo);

        let request = Request::builder()
            .method("GET")
            .uri("/expenses/latest")
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<ShowLatestExpense>(&body_bytes).unwrap();

        assert_eq!(body, show_latest_expense_response());
    }

    #[tokio::test]
    async fn test_update_handler() {
        // Prepare
        let repo = MockExpenseRepository::new();
        let app = expense_routes().with_state(repo);

        // Use serde_json::json! macro to avoid serialization issues
        let request = Request::builder()
            .method("PUT")
            .uri("/expenses/1")
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "amount": 1000,
                    "date": "2025-04-01",
                    "description": "Updated test expense",
                    "priority": 1,
                    "categoryId": 1,
                    "walletId": 1,
                    "tagIds": [1, 2]
                })
                .to_string(),
            ))
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
