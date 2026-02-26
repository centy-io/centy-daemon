use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssetScope { #[default] IssueSpecific, Shared }

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
    #[error("IO error: {0}")] IoError(#[from] std::io::Error),
    #[error("Manifest error: {0}")] ManifestError(#[from] crate::manifest::ManifestError),
    #[error("Centy not initialized. Run 'centy init' first.")] NotInitialized,
    #[error("Issue not found: {0}")] IssueNotFound(String),
    #[error("Asset not found: {0}")] AssetNotFound(String),
    #[error("Asset already exists: {0}")] AssetAlreadyExists(String),
    #[error("Invalid filename: {0}")] InvalidFilename(String),
    #[error("Unsupported file type: {0}")] UnsupportedFileType(String),
}

#[derive(Debug, Clone)]
pub struct AddAssetResult { pub asset: AssetInfo, pub path: String }

#[derive(Debug, Clone)]
pub struct DeleteAssetResult { pub filename: String, pub was_shared: bool }

pub const IMAGE_MIME_TYPES: &[(&str, &str)] = &[
    ("png", "image/png"), ("jpg", "image/jpeg"), ("jpeg", "image/jpeg"),
    ("gif", "image/gif"), ("webp", "image/webp"), ("svg", "image/svg+xml"),
    ("ico", "image/x-icon"), ("bmp", "image/bmp"),
];

pub const VIDEO_MIME_TYPES: &[(&str, &str)] = &[
    ("mp4", "video/mp4"), ("webm", "video/webm"), ("mov", "video/quicktime"),
    ("avi", "video/x-msvideo"), ("mkv", "video/x-matroska"),
];

pub fn compute_binary_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn get_mime_type(filename: &str) -> Option<String> {
    let extension = filename.rsplit('.').next()?.to_lowercase();
    for (ext, mime) in IMAGE_MIME_TYPES {
        if extension == *ext { return Some((*mime).to_string()); }
    }
    for (ext, mime) in VIDEO_MIME_TYPES {
        if extension == *ext { return Some((*mime).to_string()); }
    }
    None
}

pub fn sanitize_filename(filename: &str) -> Result<String, AssetError> {
    if filename.is_empty() {
        return Err(AssetError::InvalidFilename("Filename cannot be empty".to_string()));
    }
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(AssetError::InvalidFilename(
            "Filename cannot contain path separators or '..'".to_string(),
        ));
    }
    if filename.starts_with('.') {
        return Err(AssetError::InvalidFilename("Filename cannot start with '.'".to_string()));
    }
    if filename.len() > 255 {
        return Err(AssetError::InvalidFilename(
            "Filename too long (max 255 characters)".to_string(),
        ));
    }
    Ok(filename.to_string())
}
