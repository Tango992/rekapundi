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
    dtos::income::{IndexIncomeQuery, IndexIncomeResponse, SaveBatchIncome, SaveIncome},
    repositories::income,
};

/// Handles the routes related to incomes operations.
pub fn income_routes() -> Router<Arc<dyn income::RepositoryOperation>> {
    Router::new().nest(
        "/incomes",
        Router::new()
            .route("/", get(index))
            .route("/", post(save_bulk))
            .route("/{id}", get(show))
            .route("/{id}", put(update))
            .route("/{id}", delete(destroy))
            .route("/latest", get(show_latest)),
    )
}

/// Handles the deletion of a specific income by ID.
async fn destroy(
    WithRejection(Path(id), _): WithRejection<Path<u32>, AppError>,
    State(income_repository): State<Arc<dyn income::RepositoryOperation>>,
) -> Result<impl IntoResponse, AppError> {
    income_repository.delete(id as i32).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Handles showing the list of incomes.
async fn index(
    Query(query): Query<IndexIncomeQuery>,
    State(income_repository): State<Arc<dyn income::RepositoryOperation>>,
) -> Result<impl IntoResponse, AppError> {
    let incomes = income_repository.find_all(&query).await?;

    Ok((StatusCode::OK, Json(IndexIncomeResponse { incomes })))
}

/// Handles the bulk save of incomes.
async fn save_bulk(
    State(income_repository): State<Arc<dyn income::RepositoryOperation>>,
    WithRejection(Json(body), _): WithRejection<Json<SaveBatchIncome>, AppError>,
) -> Result<impl IntoResponse, AppError> {
    income_repository.insert_bulk(body.incomes).await?;

    Ok(StatusCode::CREATED)
}

/// Handles the retrieval of a specific income by ID.
async fn show(
    WithRejection(Path(id), _): WithRejection<Path<u32>, AppError>,
    State(income_repository): State<Arc<dyn income::RepositoryOperation>>,
) -> Result<impl IntoResponse, AppError> {
    let income = income_repository.find_one(id as i32).await?;

    Ok((StatusCode::OK, Json(income)))
}

/// Handles the retrieval of the latest income.
async fn show_latest(
    State(income_repository): State<Arc<dyn income::RepositoryOperation>>,
) -> Result<impl IntoResponse, AppError> {
    let latest_income = income_repository.find_latest().await?;

    Ok((StatusCode::OK, Json(latest_income)))
}

/// Handles the update of a specific income by ID.
async fn update(
    WithRejection(Path(id), _): WithRejection<Path<u32>, AppError>,
    State(income_repository): State<Arc<dyn income::RepositoryOperation>>,
    WithRejection(Json(body), _): WithRejection<Json<SaveIncome>, AppError>,
) -> Result<impl IntoResponse, AppError> {
    income_repository.update(id as i32, &body).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dtos::{
        income::{IndexIncomeQuery, SaveIncome},
        query_result::{IndexIncomeElement, ShowIncome, ShowLatestIncome, SimpleEntity},
    };

    use async_trait::async_trait;
    use axum::{
        body::{Body, to_bytes},
        extract::Request,
        http::StatusCode,
    };
    use serde_json;
    use sqlx::Error as SqlxError;
    use std::sync::Arc;
    use tower::{Service, ServiceExt};

    pub struct MockIncomeRepository;

    impl MockIncomeRepository {
        pub fn new() -> Arc<Self> {
            Arc::new(Self)
        }
    }

    fn index_income_response() -> IndexIncomeResponse {
        IndexIncomeResponse {
            incomes: vec![
                IndexIncomeElement {
                    id: 1,
                    amount: 5000,
                    date: "2025-04-01".to_string(),
                    description: Some("Test income 1".to_string()),
                },
                IndexIncomeElement {
                    id: 2,
                    amount: 3000,
                    date: "2025-04-02".to_string(),
                    description: None,
                },
            ],
        }
    }

    fn show_latest_income_response() -> ShowLatestIncome {
        ShowLatestIncome {
            id: 3,
            amount: 8000,
            date: "2025-04-03".to_string(),
            description: Some("Latest test income".to_string()),
            wallet: sqlx::types::Json(SimpleEntity {
                id: 1,
                name: "Bank Account".to_string(),
            }),
        }
    }

    fn show_income_response(id: i32) -> ShowIncome {
        ShowIncome {
            amount: 5000,
            date: "2025-04-01".to_string(),
            description: Some(format!("Test income {}", id)),
            wallet: sqlx::types::Json(SimpleEntity {
                id: 1,
                name: "Bank Account".to_string(),
            }),
        }
    }

    #[async_trait]
    impl income::RepositoryOperation for MockIncomeRepository {
        async fn delete(&self, _id: i32) -> Result<(), SqlxError> {
            Ok(())
        }

        async fn find_all(
            &self,
            _query: &IndexIncomeQuery,
        ) -> Result<Vec<IndexIncomeElement>, SqlxError> {
            Ok(index_income_response().incomes)
        }

        async fn find_latest(&self) -> Result<ShowLatestIncome, SqlxError> {
            Ok(show_latest_income_response())
        }

        async fn find_one(&self, id: i32) -> Result<ShowIncome, SqlxError> {
            Ok(show_income_response(id))
        }

        async fn insert_bulk(&self, _incomes: Vec<SaveIncome>) -> Result<(), SqlxError> {
            Ok(())
        }

        async fn update(&self, _id: i32, _income: &SaveIncome) -> Result<(), SqlxError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_destroy_handler() {
        // Prepare
        let repo = MockIncomeRepository::new();

        let mut app = income_routes().with_state(repo).into_service();

        let request = Request::builder()
            .method("DELETE")
            .uri("/incomes/1")
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_index_handler() {
        // Prepare
        let repo = MockIncomeRepository::new();

        let mut app = income_routes().with_state(repo).into_service();

        let request = Request::builder()
            .method("GET")
            .uri("/incomes")
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<IndexIncomeResponse>(&body_bytes).unwrap();

        assert_eq!(body, index_income_response());
    }

    #[tokio::test]
    async fn test_save_bulk_handler() {
        // Prepare
        let repo = MockIncomeRepository::new();

        let mut app = income_routes().with_state(repo).into_service();

        // Use serde_json::json! macro to create request body
        let request = Request::builder()
            .method("POST")
            .uri("/incomes")
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "incomes": [{
                        "amount": 5000,
                        "date": "2025-04-01",
                        "description": "Test income",
                        "walletId": 1
                    }]
                })
                .to_string(),
            ))
            .unwrap();

        // Execute
        let response = ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_show_handler() {
        // Prepare
        let repo = MockIncomeRepository::new();

        let mut app = income_routes().with_state(repo).into_service();

        let request = Request::builder()
            .method("GET")
            .uri("/incomes/1")
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<ShowIncome>(&body_bytes).unwrap();

        assert_eq!(body, show_income_response(1));
    }

    #[tokio::test]
    async fn test_show_latest_handler() {
        // Prepare
        let repo = MockIncomeRepository::new();

        let mut app = income_routes().with_state(repo).into_service();

        let request = Request::builder()
            .method("GET")
            .uri("/incomes/latest")
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<ShowLatestIncome>(&body_bytes).unwrap();

        assert_eq!(body, show_latest_income_response());
    }

    #[tokio::test]
    async fn test_update_handler() {
        // Prepare
        let repo = MockIncomeRepository::new();

        let mut app = income_routes().with_state(repo).into_service();

        // Use serde_json::json! macro to create request body
        let request = Request::builder()
            .method("PUT")
            .uri("/incomes/1")
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "amount": 5000,
                    "date": "2025-04-01",
                    "description": "Updated test income",
                    "walletId": 1
                })
                .to_string(),
            ))
            .unwrap();

        // Execute
        let response = ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
