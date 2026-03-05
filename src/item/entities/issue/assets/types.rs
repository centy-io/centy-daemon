use thiserror::Error;

pub use super::helpers::{
    compute_binary_hash, get_mime_type, sanitize_filename, IMAGE_MIME_TYPES, VIDEO_MIME_TYPES,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssetScope {
    #[default]
    IssueSpecific,
    Shared,
}

#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub filename: String,
    pub hash: String,
    pub size: u64,
    pub mime_type: String,
    pub is_shared: bool,
    pub created_at: String,
}

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),
    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,
    #[error("Issue not found: {0}")]
    IssueNotFound(String),
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
    #[error("Asset already exists: {0}")]
    AssetAlreadyExists(String),
    #[error("Invalid filename: {0}")]
    InvalidFilename(String),
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
}

#[derive(Debug, Clone)]
pub struct AddAssetResult {
    pub asset: AssetInfo,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct DeleteAssetResult {
    pub filename: String,
    pub was_shared: bool,
}
