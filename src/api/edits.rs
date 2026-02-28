use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::error::AppError;
use crate::models::EditParams;
use crate::state::AppState;
use crate::storage;

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/v1/images/{id}/edits",
        get(get_edits).put(update_edits),
    )
}

async fn get_edits(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<EditParams>, AppError> {
    storage::ensure_image_exists(&state, &id)?;
    let edits = storage::load_edits(&state, &id).await?;
    Ok(Json(edits))
}

async fn update_edits(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(incoming): Json<EditParams>,
) -> Result<Json<EditParams>, AppError> {
    storage::ensure_image_exists(&state, &id)?;

    let mut edits = storage::load_edits(&state, &id).await?;
    edits.merge(&incoming);
    storage::save_edits(&state, &id, &edits).await?;

    // Clear preview cache since edits changed
    storage::clear_cache(&state, &id).await?;

    tracing::info!(id = %id, "Edits updated");
    Ok(Json(edits))
}
