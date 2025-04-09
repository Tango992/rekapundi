mod common;
mod constants;
mod dtos;
mod handlers;
mod repositories;

use axum::{Router, http::StatusCode, routing::get};
use common::trace::http_trace_layer;
use handlers::{expense::expense_routes, income::income_routes, util::util_routes};
use repositories::{expense, income, util};
use std::{env, sync::Arc};
use tower_http::compression::CompressionLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    common::trace::init();
    let pg_pool = Arc::new(common::database::init().await.unwrap());

    let expense_repository = Arc::new(expense::Repository::new(Arc::clone(&pg_pool)));
    let income_repository = Arc::new(income::Repository::new(Arc::clone(&pg_pool)));
    let util_repository = Arc::new(util::Repository::new(Arc::clone(&pg_pool)));

    let app = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .merge(expense_routes().with_state(expense_repository))
        .merge(income_routes().with_state(income_repository))
        .merge(util_routes().with_state(util_repository))
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
