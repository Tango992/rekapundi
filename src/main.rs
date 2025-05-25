mod common;
mod constants;
mod dtos;
mod entities;
mod handlers;
mod middlewares;
mod repositories;
mod services;

use axum::{Router, http::StatusCode, middleware, routing::get};
use handlers::{
    expense::expense_routes, income::income_routes, summary::summary_routes, util::util_routes,
    wallet::wallet_routes,
};
use middlewares::{auth::authenticate_request, trace::http_trace_layer};
use repositories::{expense, income, summary, util};
use std::{env, sync::Arc};
use tower_http::compression::CompressionLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    middlewares::trace::init();
    let pg_pool = Arc::new(common::database::init().await.unwrap());

    let expense_repository = Arc::new(expense::Repository::new(Arc::clone(&pg_pool)));
    let income_repository = Arc::new(income::Repository::new(Arc::clone(&pg_pool)));
    let summary_repository = Arc::new(summary::SummaryRepository::new(Arc::clone(&pg_pool)));
    let util_repository = Arc::new(util::Repository::new(Arc::clone(&pg_pool)));
    let wallet_repository = Arc::new(repositories::wallet::Repository::new(Arc::clone(&pg_pool)));

    let auth_required_router = Router::new()
        .merge(expense_routes().with_state(expense_repository))
        .merge(income_routes().with_state(income_repository))
        .merge(summary_routes().with_state(summary_repository))
        .merge(util_routes().with_state(util_repository))
        .merge(wallet_routes().with_state(wallet_repository))
        .route_layer(middleware::from_fn(authenticate_request));

    let app = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .merge(auth_required_router)
        .layer(CompressionLayer::new())
        .layer(http_trace_layer());

    let port = env::var("PORT")
        .inspect_err(|_| {
            tracing::error!("PORT not found in environment");
        })
        .unwrap();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!(
        "Listening on port {}",
        listener.local_addr().unwrap().port()
    );
    axum::serve(listener, app).await.unwrap();
}
