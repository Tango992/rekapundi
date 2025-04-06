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
    State(util_repository): State<Arc<UtilRepository>>,
    Query(query): Query<Pagination>,
) -> Result<impl IntoResponse, AppError> {
    let categories = util_repository
        .find_many_categories(query.offset(), query.limit())
        .await?;

    Ok((StatusCode::OK, Json(IndexCategoriesResponse { categories })))
}

/// Handler to list all parent categories.
async fn index_parent_categories(
    State(util_repository): State<Arc<UtilRepository>>,
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
    State(util_repository): State<Arc<UtilRepository>>,
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
    State(util_repository): State<Arc<UtilRepository>>,
    Query(query): Query<Pagination>,
) -> Result<impl IntoResponse, AppError> {
    let wallets = util_repository
        .find_many_wallets(query.offset(), query.limit())
        .await?;

    Ok((StatusCode::OK, Json(IndexWalletsResponse { wallets })))
}
