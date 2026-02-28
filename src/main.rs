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
    let state = AppState::new(data_dir);
    storage::init_storage(&state)
        .await
        .expect("Failed to initialize storage");

    let app = Router::new()
        .route("/health", get(health))
        .merge(api::router())
        .layer(RequestBodyLimitLayer::new(200 * 1024 * 1024)) // 200MB
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    tracing::info!("Hilaw server listening on http://{addr}");
    axum::serve(listener, app).await.expect("Server error");
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
