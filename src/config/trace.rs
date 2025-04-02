use tower_http::trace::{self, HttpMakeClassifier, TraceLayer};

/// Initialize tracing subscriber.
pub fn init() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .compact()
        .init();
}

/// This module provides a function to create a `TraceLayer` for HTTP requests.
/// It uses the `tower_http` crate to create a layer that can be used with a `tower` service.
pub fn http_trace_layer() -> TraceLayer<HttpMakeClassifier> {
    TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().include_headers(false))
        .on_request(trace::DefaultOnRequest::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO))
        .on_failure(trace::DefaultOnFailure::new().level(tracing::Level::ERROR))
}
