mod config;
use axum::{Router, http::StatusCode, routing::get};
use config::trace::http_trace_layer;
use std::env;
use tower_http::compression::CompressionLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    config::trace::init();

    let app = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
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
