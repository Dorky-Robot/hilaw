use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::Response;
use axum::routing::get;
use axum::Router;

use crate::error::AppError;
use crate::models::PreviewQuery;
use crate::processing::pipeline;
use crate::state::AppState;
use crate::storage;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/images/{id}/preview", get(preview))
}

async fn preview(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<PreviewQuery>,
) -> Result<Response, AppError> {
    storage::ensure_image_exists(&state, &id)?;

    let cache_path = storage::cache_path(&state, &id, query.width, query.height);
    if cache_path.exists() {
        let bytes = tokio::fs::read(&cache_path).await?;
        return Ok(jpeg_response(bytes));
    }

    let raw_path = storage::original_path(&state, &id).await?;
    let edits = storage::load_edits(&state, &id).await?;
    let width = query.width;
    let height = query.height;

    let jpeg_bytes = tokio::task::spawn_blocking(move || {
        let img = pipeline::process_raw(&raw_path, &edits, width as usize, height as usize)?;
        pipeline::encode_jpeg(&img, 85)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

    tokio::fs::write(&cache_path, &jpeg_bytes).await?;

    Ok(jpeg_response(jpeg_bytes))
}

fn jpeg_response(bytes: Vec<u8>) -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, "image/jpeg")
        .body(Body::from(bytes))
        .unwrap()
}
