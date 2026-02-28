use axum::extract::{Multipart, State};
use axum::routing::post;
use axum::{Json, Router};
use chrono::Utc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{EditParams, ImageRecord};
use crate::state::AppState;
use crate::storage;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/images", post(upload))
}

async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ImageRecord>, AppError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?
        .ok_or_else(|| AppError::BadRequest("No file field in upload".into()))?;

    let filename = field
        .file_name()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown.raw".into());

    let ext = storage::validate_raw_extension(&filename)?;
    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read upload: {e}")))?;

    let id = Uuid::now_v7().to_string();
    storage::create_image_dir(&state, &id).await?;

    let original_path = state.image_dir(&id).join(format!("original.{ext}"));
    tokio::fs::write(&original_path, &data).await?;

    let record = ImageRecord {
        id: id.clone(),
        filename,
        extension: ext,
        size_bytes: data.len() as u64,
        created_at: Utc::now(),
    };

    storage::save_meta(&state, &record).await?;
    storage::save_edits(&state, &id, &EditParams::default()).await?;

    tracing::info!(id = %id, "Image uploaded");
    Ok(Json(record))
}
