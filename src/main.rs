mod common;
mod repositories;
use axum::{Router, http::StatusCode, routing::get};
use common::trace::http_trace_layer;
use std::env;
use tower_http::compression::CompressionLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    common::trace::init();
    // The PostgreSQL connection pool.
    let pg_pool = common::database::init().await.unwrap();

    let app = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .with_state(pg_pool)
        .layer(CompressionLayer::new())
        .layer(http_trace_layer());

    let port = env::var("PORT").expect("PORT is not set");
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!(
        "Listening on port {}",
        listener.local_addr().unwrap().port()
    );
    axum::serve(listener, app).await.unwrap();
}
