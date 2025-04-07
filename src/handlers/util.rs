use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use std::sync::Arc;

use crate::{
    common::errors::AppError,
    dtos::{
        Pagination,
        util::{
            IndexCategoriesResponse, IndexParentCategoriesResponse, IndexTagsQuery,
            IndexTagsResponse, IndexWalletsResponse,
        },
    },
    repositories::util::{UtilOperation, UtilRepository},
};

pub fn util_routes() -> Router<Arc<UtilRepository>> {
    Router::new()
        .route("/categories", get(index_categories))
        .route("/parent-categories", get(index_parent_categories))
        .route("/tags", get(index_tags))
        .route("/wallets", get(index_wallets))
}

/// Handler to list all categories.
async fn index_categories(
    State(util_repository): State<Arc<impl UtilOperation>>,
    Query(query): Query<Pagination>,
) -> Result<impl IntoResponse, AppError> {
    let categories = util_repository
        .find_many_categories(query.offset(), query.limit())
        .await?;

    Ok((StatusCode::OK, Json(IndexCategoriesResponse { categories })))
}

/// Handler to list all parent categories.
async fn index_parent_categories(
    State(util_repository): State<Arc<impl UtilOperation>>,
    Query(query): Query<Pagination>,
) -> Result<impl IntoResponse, AppError> {
    let parent_categories = util_repository
        .find_many_parent_categories(query.offset(), query.limit())
        .await?;

    Ok((
        StatusCode::OK,
        Json(IndexParentCategoriesResponse { parent_categories }),
    ))
}

/// Handler to list all tags.
async fn index_tags(
    State(util_repository): State<Arc<impl UtilOperation>>,
    Query(query): Query<IndexTagsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let tags = util_repository
        .find_many_tags(
            query.mark_important_value,
            query.pagination.offset(),
            query.pagination.limit(),
        )
        .await?;

    Ok((StatusCode::OK, Json(IndexTagsResponse { tags })))
}

/// Handler to list all wallets.
async fn index_wallets(
    State(util_repository): State<Arc<impl UtilOperation>>,
    Query(query): Query<Pagination>,
) -> Result<impl IntoResponse, AppError> {
    let wallets = util_repository
        .find_many_wallets(query.offset(), query.limit())
        .await?;

    Ok((StatusCode::OK, Json(IndexWalletsResponse { wallets })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dtos::query_result::{ParentCategory, SimpleEntity, Tag};
    use async_trait::async_trait;
    use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};
    use serde_json;
    use sqlx::Error as SqlxError;
    use std::sync::Arc;

    pub struct MockUtilRepository;

    impl MockUtilRepository {
        pub fn new() -> Arc<Self> {
            Arc::new(Self)
        }
    }

    fn categories_response() -> Vec<SimpleEntity> {
        vec![
            SimpleEntity {
                id: 1,
                name: "Food".to_string(),
            },
            SimpleEntity {
                id: 2,
                name: "Transportation".to_string(),
            },
        ]
    }

    fn parent_categories_response() -> Vec<ParentCategory> {
        vec![
            ParentCategory {
                id: 1,
                name: "Daily Expenses".to_string(),
                categories: sqlx::types::Json(vec![
                    SimpleEntity {
                        id: 1,
                        name: "Food".to_string(),
                    },
                    SimpleEntity {
                        id: 2,
                        name: "Transportation".to_string(),
                    },
                ]),
            },
            ParentCategory {
                id: 2,
                name: "Monthly Bills".to_string(),
                categories: sqlx::types::Json(vec![
                    SimpleEntity {
                        id: 3,
                        name: "Rent".to_string(),
                    },
                    SimpleEntity {
                        id: 4,
                        name: "Utilities".to_string(),
                    },
                ]),
            },
        ]
    }

    fn tags_response() -> Vec<Tag> {
        vec![
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
        ]
    }

    fn wallets_response() -> Vec<SimpleEntity> {
        vec![
            SimpleEntity {
                id: 1,
                name: "Cash".to_string(),
            },
            SimpleEntity {
                id: 2,
                name: "Credit Card".to_string(),
            },
        ]
    }

    #[async_trait]
    impl UtilOperation for MockUtilRepository {
        async fn find_many_categories(
            &self,
            _offset: i64,
            _limit: i64,
        ) -> Result<Vec<SimpleEntity>, SqlxError> {
            Ok(categories_response())
        }

        async fn find_many_parent_categories(
            &self,
            _offset: i64,
            _limit: i64,
        ) -> Result<Vec<ParentCategory>, SqlxError> {
            Ok(parent_categories_response())
        }

        async fn find_many_tags(
            &self,
            _mark_important_value: Option<bool>,
            _offset: i64,
            _limit: i64,
        ) -> Result<Vec<Tag>, SqlxError> {
            Ok(tags_response())
        }

        async fn find_many_wallets(
            &self,
            _offset: i64,
            _limit: i64,
        ) -> Result<Vec<SimpleEntity>, SqlxError> {
            Ok(wallets_response())
        }
    }

    #[tokio::test]
    async fn test_index_categories_handler() {
        // Prepare
        let repo = MockUtilRepository::new();
        let pagination = Pagination::default();

        // Execute
        let result = index_categories(State(repo), Query(pagination)).await;

        // Assert
        assert!(result.is_ok());
        let response = result.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<IndexCategoriesResponse>(&body_bytes).unwrap();

        assert_eq!(body.categories, categories_response());
    }

    #[tokio::test]
    async fn test_index_parent_categories_handler() {
        // Prepare
        let repo = MockUtilRepository::new();
        let pagination = Pagination::default();

        // Execute
        let result = index_parent_categories(State(repo), Query(pagination)).await;

        // Assert
        assert!(result.is_ok());
        let response = result.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<IndexParentCategoriesResponse>(&body_bytes).unwrap();

        assert_eq!(body.parent_categories, parent_categories_response());
    }

    #[tokio::test]
    async fn test_index_tags_handler() {
        // Prepare
        let repo = MockUtilRepository::new();
        let query = IndexTagsQuery {
            mark_important_value: Some(true),
            pagination: Pagination::default(),
        };

        // Execute
        let result = index_tags(State(repo), Query(query)).await;

        // Assert
        assert!(result.is_ok());
        let response = result.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<IndexTagsResponse>(&body_bytes).unwrap();

        assert_eq!(body.tags, tags_response());
    }

    #[tokio::test]
    async fn test_index_wallets_handler() {
        // Prepare
        let repo = MockUtilRepository::new();
        let pagination = Pagination::default();

        // Execute
        let result = index_wallets(State(repo), Query(pagination)).await;

        // Assert
        assert!(result.is_ok());
        let response = result.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = serde_json::from_slice::<IndexWalletsResponse>(&body_bytes).unwrap();

        assert_eq!(body.wallets, wallets_response());
    }
}
