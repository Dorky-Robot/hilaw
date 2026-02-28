use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::models::ImageRecord;
use crate::state::AppState;
use crate::storage;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/images", get(list_images))
        .route("/api/v1/images/{id}", get(get_image).delete(delete_image))
}

async fn list_images(State(state): State<AppState>) -> Result<Json<Vec<ImageRecord>>, AppError> {
    let records = storage::list_images(&state).await?;
    Ok(Json(records))
}

async fn get_image(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let meta = storage::load_meta(&state, &id).await?;
    let edits = storage::load_edits(&state, &id).await?;
    Ok(Json(json!({
        "image": meta,
        "edits": edits,
    })))
}

async fn delete_image(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    storage::delete_image(&state, &id).await?;
    tracing::info!(id = %id, "Image deleted");
    Ok(Json(json!({ "deleted": id })))
}
