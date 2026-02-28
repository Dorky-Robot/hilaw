use std::path::Path;

use crate::error::AppError;
use crate::models::{EditParams, ImageRecord};
use crate::state::AppState;

pub async fn init_storage(state: &AppState) -> Result<(), AppError> {
    tokio::fs::create_dir_all(state.images_dir()).await?;
    Ok(())
}

pub async fn create_image_dir(state: &AppState, id: &str) -> Result<(), AppError> {
    let dir = state.image_dir(id);
    tokio::fs::create_dir_all(dir.join("cache")).await?;
    Ok(())
}

pub async fn save_meta(state: &AppState, record: &ImageRecord) -> Result<(), AppError> {
    let path = state.image_dir(&record.id).join("meta.json");
    let json = serde_json::to_string_pretty(record)?;
    tokio::fs::write(path, json).await?;
    Ok(())
}

pub async fn load_meta(state: &AppState, id: &str) -> Result<ImageRecord, AppError> {
    let path = state.image_dir(id).join("meta.json");
    if !path.exists() {
        return Err(AppError::NotFound(format!("Image {id} not found")));
    }
    let data = tokio::fs::read_to_string(path).await?;
    Ok(serde_json::from_str(&data)?)
}

pub async fn save_edits(state: &AppState, id: &str, edits: &EditParams) -> Result<(), AppError> {
    let path = state.image_dir(id).join("edits.json");
    let json = serde_json::to_string_pretty(edits)?;
    tokio::fs::write(path, json).await?;
    Ok(())
}

pub async fn load_edits(state: &AppState, id: &str) -> Result<EditParams, AppError> {
    let path = state.image_dir(id).join("edits.json");
    if !path.exists() {
        return Ok(EditParams::default());
    }
    let data = tokio::fs::read_to_string(path).await?;
    Ok(serde_json::from_str(&data)?)
}

pub async fn original_path(state: &AppState, id: &str) -> Result<std::path::PathBuf, AppError> {
    let meta = load_meta(state, id).await?;
    let path = state
        .image_dir(id)
        .join(format!("original.{}", meta.extension));
    Ok(path)
}

pub async fn list_images(state: &AppState) -> Result<Vec<ImageRecord>, AppError> {
    let images_dir = state.images_dir();
    if !images_dir.exists() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    let mut entries = tokio::fs::read_dir(&images_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            let meta_path = entry.path().join("meta.json");
            if meta_path.exists() {
                let data = tokio::fs::read_to_string(&meta_path).await?;
                if let Ok(record) = serde_json::from_str::<ImageRecord>(&data) {
                    records.push(record);
                }
            }
        }
    }
    records.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(records)
}

pub async fn delete_image(state: &AppState, id: &str) -> Result<(), AppError> {
    let dir = state.image_dir(id);
    if !dir.exists() {
        return Err(AppError::NotFound(format!("Image {id} not found")));
    }
    tokio::fs::remove_dir_all(dir).await?;
    Ok(())
}

pub async fn clear_cache(state: &AppState, id: &str) -> Result<(), AppError> {
    let cache_dir = state.image_dir(id).join("cache");
    if cache_dir.exists() {
        tokio::fs::remove_dir_all(&cache_dir).await?;
        tokio::fs::create_dir_all(&cache_dir).await?;
    }
    Ok(())
}

pub fn cache_path(state: &AppState, id: &str, width: u32, height: u32) -> std::path::PathBuf {
    state
        .image_dir(id)
        .join("cache")
        .join(format!("preview_{width}x{height}.jpg"))
}

pub fn ensure_image_exists(state: &AppState, id: &str) -> Result<(), AppError> {
    if !state.image_dir(id).exists() {
        return Err(AppError::NotFound(format!("Image {id} not found")));
    }
    Ok(())
}

/// Extract file extension, validating it's a known RAW format
pub fn validate_raw_extension(filename: &str) -> Result<String, AppError> {
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| AppError::BadRequest("File has no extension".into()))?;

    let raw_extensions = [
        "cr2", "cr3", "nef", "arw", "orf", "rw2", "dng", "raf", "pef", "srw", "x3f", "3fr",
        "mrw", "nrw", "raw",
    ];

    if !raw_extensions.contains(&ext.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Unsupported file format: .{ext}"
        )));
    }

    Ok(ext)
}
