use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use axum_extra::extract::WithRejection;
use std::sync::Arc;
use validator::Validate;

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
    body.validate()?;

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
    body.validate()?;

    expense_repository.update(id as i32, &body).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dtos::{
            Pagination,
            expense::{IndexExpenseQuery, SaveBatchExpense, SaveExpense},
            query_result::{
                IndexExpenseElement, ShowExpense, ShowLatestExpense, SimpleEntity, Tag,
            },
        },
        handlers::expense::{destroy, index, save_bulk, show, show_latest, update},
    };

    use async_trait::async_trait;
    use axum::{
        Json,
        body::to_bytes,
        extract::{Path, Query, State},
        http::StatusCode,
        response::IntoResponse,
    };
    use axum_extra::extract::WithRejection;
    use serde_json;
    use sqlx::Error as SqlxError;
    use std::{marker::PhantomData, sync::Arc};
    use time::Date;

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

        async fn insert_bulk(&self, _expenses: &Vec<SaveExpense>) -> Result<(), SqlxError> {
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

        // Execute
        let result = destroy(
            WithRejection(Path(1), PhantomData::<crate::handlers::expense::AppError>),
            State(repo),
        )
        .await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.into_response().status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_index_handler() {
        // Prepare
        let repo = MockExpenseRepository::new();
        let query = IndexExpenseQuery {
            start_date: None,
            end_date: None,
            pagination: Pagination::default(),
        };

        // Execute
        let result = index(Query(query), State(repo)).await;

        // Assert
        assert!(result.is_ok());

        let response = result.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<IndexExpenseResponse>(&body_bytes).unwrap();

        assert_eq!(body, index_expense_response());
    }

    #[tokio::test]
    async fn test_save_bulk_handler() {
        // Prepare
        let repo = MockExpenseRepository::new();

        let bulk_expense = SaveBatchExpense {
            expenses: vec![SaveExpense {
                amount: 1000,
                date: Date::from_calendar_date(2025, time::Month::April, 1).unwrap(),
                description: Some("Test expense".to_string()),
                priority: 1,
                category_id: 1,
                wallet_id: 1,
                tag_ids: vec![1, 2],
            }],
        };

        // Execute
        let result = save_bulk(
            State(repo),
            WithRejection(
                Json(bulk_expense),
                PhantomData::<crate::handlers::expense::AppError>,
            ),
        )
        .await;

        // Assert
        assert!(result.is_ok());

        let response = result.into_response();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_show_handler() {
        // Prepare
        let repo = MockExpenseRepository::new();
        let with_rejection =
            WithRejection(Path(1), PhantomData::<crate::handlers::expense::AppError>);

        // Execute
        let result = show(with_rejection, State(repo)).await;

        // Assert
        assert!(result.is_ok());
        let response = result.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<ShowExpense>(&body_bytes).unwrap();

        assert_eq!(body, show_expense_response(1));
    }

    #[tokio::test]
    async fn test_show_latest_handler() {
        // Prepare
        let repo = MockExpenseRepository::new();

        // Execute
        let result = show_latest(State(repo)).await;

        // Assert
        assert!(result.is_ok());
        let response = result.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<ShowLatestExpense>(&body_bytes).unwrap();

        assert_eq!(body, show_latest_expense_response());
    }

    #[tokio::test]
    async fn test_update_handler() {
        // Prepare
        let repo = MockExpenseRepository::new();
        let with_rejection =
            WithRejection(Path(1), PhantomData::<crate::handlers::expense::AppError>);

        let update_expense = SaveExpense {
            amount: 1000,
            date: Date::from_calendar_date(2025, time::Month::April, 1).unwrap(),
            description: Some("Updated test expense".to_string()),
            priority: 1,
            category_id: 1,
            wallet_id: 1,
            tag_ids: vec![1, 2],
        };

        // Execute
        let result = update(
            with_rejection,
            State(repo),
            WithRejection(
                Json(update_expense),
                PhantomData::<crate::handlers::expense::AppError>,
            ),
        )
        .await;

        // Assert
        assert!(result.is_ok());
        let response = result.into_response();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
