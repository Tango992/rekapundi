use crate::{
    common::errors::AppError,
    dtos::{
        Pagination,
        wallet::{
            IndexWalletsResponse, SaveMoneyTransfer, SaveMoneyTransferFee, SaveMoneyTransferRequest,
        },
    },
    repositories::wallet,
};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::WithRejection;
use std::sync::Arc;

pub fn wallet_routes() -> Router<Arc<dyn wallet::RepositoryOperation>> {
    Router::new().nest(
        "/wallets",
        Router::new()
            .route("/", get(index))
            .route("/transfer", post(transfer)),
    )
}

async fn index(
    State(wallet_repository): State<Arc<dyn wallet::RepositoryOperation>>,
    Query(query): Query<Pagination>,
) -> Result<impl IntoResponse, AppError> {
    let wallets = wallet_repository
        .find_many(query.offset(), query.limit())
        .await?;

    Ok((StatusCode::OK, Json(IndexWalletsResponse { wallets })))
}

async fn transfer(
    State(wallet_repository): State<Arc<dyn wallet::RepositoryOperation>>,
    WithRejection(Json(body), _): WithRejection<Json<SaveMoneyTransferRequest>, AppError>,
) -> Result<impl IntoResponse, AppError> {
    let money_transfer = SaveMoneyTransfer {
        source_wallet_id: body.source_wallet_id,
        target_wallet_id: body.target_wallet_id,
        amount: body.amount,
        date: body.date,
        description: body.description.clone(),
    };

    let save_fee = match body.fee {
        0 => None,
        _ => {
            let description = match body.description {
                Some(desc) => Some(format!("Wallet transfer fee: {}", desc)),
                None => None,
            };

            Some(SaveMoneyTransferFee {
                priority: 2, // Default to secondary priority
                wallet_id: body.source_wallet_id,
                category_id: 1, // Default to a specific category ID
                amount: body.fee,
                date: body.date,
                description,
            })
        }
    };

    wallet_repository
        .insert_money_transfer(&money_transfer, save_fee.as_ref())
        .await?;

    Ok(StatusCode::CREATED)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dtos::query_result::SimpleEntity;

    use async_trait::async_trait;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json;
    use sqlx::Error as SqlxError;
    use std::sync::Arc;
    use tower::ServiceExt;

    pub struct MockWalletRepository;

    impl MockWalletRepository {
        pub fn new() -> Arc<Self> {
            Arc::new(Self)
        }
    }

    fn index_wallets_response() -> IndexWalletsResponse {
        IndexWalletsResponse {
            wallets: vec![
                SimpleEntity {
                    id: 1,
                    name: "Cash".to_string(),
                },
                SimpleEntity {
                    id: 2,
                    name: "Bank Account".to_string(),
                },
            ],
        }
    }

    #[async_trait]
    impl wallet::RepositoryOperation for MockWalletRepository {
        async fn find_many(
            &self,
            _offset: i64,
            _limit: i64,
        ) -> Result<Vec<SimpleEntity>, SqlxError> {
            Ok(index_wallets_response().wallets)
        }

        async fn insert_money_transfer(
            &self,
            _money_transfer_record: &SaveMoneyTransfer,
            _fee_record: Option<&SaveMoneyTransferFee>,
        ) -> Result<(), SqlxError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_index_handler() {
        // Prepare
        let repo = MockWalletRepository::new();
        let app = wallet_routes().with_state(repo);

        let request = Request::builder()
            .method("GET")
            .uri("/wallets")
            .body(Body::empty())
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<IndexWalletsResponse>(&body_bytes).unwrap();

        assert_eq!(body, index_wallets_response());
    }

    #[tokio::test]
    async fn test_transfer_handler_without_fee() {
        // Prepare
        let repo = MockWalletRepository::new();
        let app = wallet_routes().with_state(repo);

        let request = Request::builder()
            .method("POST")
            .uri("/wallets/transfer")
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "sourceWalletId": 1,
                    "targetWalletId": 2,
                    "amount": 1000,
                    "fee": 0,
                    "date": "2025-05-06",
                    "description": "Test transfer without fee"
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
    async fn test_transfer_handler_with_fee() {
        // Prepare
        let repo = MockWalletRepository::new();
        let app = wallet_routes().with_state(repo);

        let request = Request::builder()
            .method("POST")
            .uri("/wallets/transfer")
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "sourceWalletId": 1,
                    "targetWalletId": 2,
                    "amount": 1000,
                    "fee": 10,
                    "date": "2025-05-06",
                    "description": "Test transfer with fee"
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
    async fn test_transfer_handler_without_description() {
        // Prepare
        let repo = MockWalletRepository::new();
        let app = wallet_routes().with_state(repo);

        let request = Request::builder()
            .method("POST")
            .uri("/wallets/transfer")
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "sourceWalletId": 1,
                    "targetWalletId": 2,
                    "amount": 1000,
                    "fee": 5,
                    "date": "2025-05-06"
                })
                .to_string(),
            ))
            .unwrap();

        // Execute
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
