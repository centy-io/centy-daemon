use thiserror::Error;
use crate::registry::RegistryError;
/// Error types for org sync operations
#[derive(Error, Debug)]
pub enum OrgSyncError {
    #[error("Registry error: {0}")] RegistryError(#[from] RegistryError),
    #[error("Sync failed: {0}")] SyncFailed(String),
    #[error("IO error: {0}")] IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")] JsonError(#[from] serde_json::Error),
    #[error("Manifest error: {0}")] ManifestError(String),
}
/// Result of syncing an org item to a single project
#[derive(Debug, Clone, serde::Serialize)]
pub struct OrgSyncResult {
    pub project_path: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
