use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecord {
    pub id: String,
    pub filename: String,
    pub extension: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditParams {
    /// Exposure compensation in EV (-5.0 to +5.0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exposure: Option<f64>,

    /// White balance temperature in Kelvin
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub white_balance: Option<f64>,

    /// Rotation in degrees (0, 90, 180, 270)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<u32>,

    /// Crop rectangle [x, y, width, height] as fractions 0.0-1.0
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<[f64; 4]>,
}

impl EditParams {
    /// Merge another EditParams on top of self (non-None fields override)
    pub fn merge(&mut self, other: &EditParams) {
        if other.exposure.is_some() {
            self.exposure = other.exposure;
        }
        if other.white_balance.is_some() {
            self.white_balance = other.white_balance;
        }
        if other.rotation.is_some() {
            self.rotation = other.rotation;
        }
        if other.crop.is_some() {
            self.crop = other.crop;
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    #[serde(default = "default_format")]
    pub format: ExportFormat,
    #[serde(default = "default_quality")]
    pub quality: u8,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Jpeg,
    Png,
}

fn default_format() -> ExportFormat {
    ExportFormat::Jpeg
}

fn default_quality() -> u8 {
    92
}

#[derive(Debug, Deserialize)]
pub struct PreviewQuery {
    #[serde(default = "default_preview_width")]
    pub width: u32,
    #[serde(default = "default_preview_height")]
    pub height: u32,
}

fn default_preview_width() -> u32 {
    800
}

fn default_preview_height() -> u32 {
    600
}
