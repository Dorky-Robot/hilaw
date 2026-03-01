use std::path::PathBuf;

use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use hilaw::api;
use hilaw::state::AppState;
use hilaw::storage;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let data_dir = PathBuf::from("data");
    let salita_url =
        std::env::var("SALITA_URL").unwrap_or_else(|_| "http://localhost:6969".into());

    let state = AppState::new(data_dir, &salita_url);
    storage::init_storage(&state)
        .await
        .expect("Failed to initialize storage");

    let app = Router::new()
        .route("/health", get(health))
        .merge(api::router())
        .layer(RequestBodyLimitLayer::new(200 * 1024 * 1024)) // 200MB
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Nest static serving: API routes take priority, fallback to static files
    let app = app.fallback_service(
        tower_http::services::ServeDir::new("static")
            .fallback(tower_http::services::ServeFile::new("static/index.html")),
    );

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    tracing::info!("Hilaw server listening on http://{addr}");
    tracing::info!("Salita endpoint: {salita_url}");
    axum::serve(listener, app).await.expect("Server error");
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
