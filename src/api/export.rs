use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::Response;
use axum::routing::post;
use axum::{Json, Router};

use crate::error::AppError;
use crate::models::{ExportFormat, ExportRequest};
use crate::processing::pipeline;
use crate::state::AppState;
use crate::storage;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/images/{id}/export", post(export))
}

async fn export(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ExportRequest>,
) -> Result<Response, AppError> {
    storage::ensure_image_exists(&state, &id)?;

    let raw_path = storage::original_path(&state, &id).await?;
    let edits = storage::load_edits(&state, &id).await?;

    let max_w = req.width.unwrap_or(0) as usize;
    let max_h = req.height.unwrap_or(0) as usize;
    let quality = req.quality;
    let format = req.format;

    let bytes = tokio::task::spawn_blocking(move || {
        let img = pipeline::process_raw(&raw_path, &edits, max_w, max_h)?;
        match format {
            ExportFormat::Jpeg => pipeline::encode_jpeg(&img, quality),
            ExportFormat::Png => pipeline::encode_png(&img),
        }
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

    let content_type = match format {
        ExportFormat::Jpeg => "image/jpeg",
        ExportFormat::Png => "image/png",
    };

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"export.{}\"", match format {
                ExportFormat::Jpeg => "jpg",
                ExportFormat::Png => "png",
            }),
        )
        .body(Body::from(bytes))
        .unwrap())
}
