use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::salita_client::DeviceInfo;
use crate::state::AppState;

#[derive(Serialize)]
struct DeviceWithDirs {
    #[serde(flatten)]
    device: DeviceInfo,
    directories: Vec<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/devices", get(list_devices))
        .route("/api/browse", get(browse_files))
}

async fn list_devices(State(state): State<AppState>) -> Result<Json<Vec<DeviceWithDirs>>, AppError> {
    let salita = state.salita();
    let devices = salita
        .list_devices()
        .await
        .map_err(|e| AppError::Internal(format!("salita error: {e}")))?;

    let mut result = Vec::new();
    for device in devices {
        let base = salita.device_url(&device);
        let dirs = match salita.get_node(&base).await {
            Ok(node) => node.directories,
            Err(_) => Vec::new(),
        };
        result.push(DeviceWithDirs {
            device,
            directories: dirs,
        });
    }

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct BrowseParams {
    pub device: String,
    pub dir: String,
    #[serde(default)]
    pub path: String,
    #[serde(default = "default_offset")]
    pub offset: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_offset() -> usize {
    0
}

fn default_limit() -> usize {
    100
}

const RAW_EXTENSIONS: &[&str] = &[
    "cr2", "cr3", "nef", "arw", "orf", "rw2", "dng", "raf", "pef", "srw", "x3f", "3fr", "mrw",
    "nrw", "raw",
];

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif", "heic", "heif"];

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "avi", "mkv", "360", "webm"];

fn classify_file(name: &str) -> &'static str {
    let ext = name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();
    if RAW_EXTENSIONS.contains(&ext.as_str()) {
        "raw"
    } else if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        "image"
    } else if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        "video"
    } else {
        "other"
    }
}

#[derive(Serialize)]
struct BrowseEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    modified: Option<String>,
    file_type: &'static str,
}

#[derive(Serialize)]
struct BrowseResponse {
    entries: Vec<BrowseEntry>,
    total: usize,
    offset: usize,
    has_more: bool,
}

async fn browse_files(
    State(state): State<AppState>,
    Query(params): Query<BrowseParams>,
) -> Result<Json<BrowseResponse>, AppError> {
    let salita = state.salita();
    let devices = salita
        .list_devices()
        .await
        .map_err(|e| AppError::Internal(format!("salita error: {e}")))?;

    let device = devices
        .iter()
        .find(|d| d.id == params.device || d.name == params.device)
        .ok_or_else(|| AppError::NotFound(format!("Device not found: {}", params.device)))?;

    let base = salita.device_url(device);
    let files = salita
        .list_files(&base, &params.dir, &params.path)
        .await
        .map_err(|e| AppError::Internal(format!("salita error: {e}")))?;

    let total = files.len();
    let entries: Vec<BrowseEntry> = files
        .into_iter()
        .skip(params.offset)
        .take(params.limit)
        .map(|f| {
            let file_type = if f.is_dir {
                "dir"
            } else {
                classify_file(&f.name)
            };
            BrowseEntry {
                name: f.name,
                path: f.path,
                is_dir: f.is_dir,
                size: f.size,
                modified: f.modified,
                file_type,
            }
        })
        .collect();

    let has_more = params.offset + params.limit < total;

    Ok(Json(BrowseResponse {
        entries,
        total,
        offset: params.offset,
        has_more,
    }))
}
