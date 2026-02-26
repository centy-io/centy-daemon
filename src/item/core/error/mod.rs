//! Unified error types for item operations.
mod impls;
use thiserror::Error;
/// Unified error type for item domain operations.
#[derive(Error, Debug)]
pub enum ItemError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Item not found: {0}")]
    NotFound(String),
    #[error("Project not initialized")]
    NotInitialized,
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("YAML error: {0}")]
    YamlError(String),
    #[error("Frontmatter error: {0}")]
    FrontmatterError(String),
    #[error("Item type not found: {0}")]
    ItemTypeNotFound(String),
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
    #[error("Item already deleted: {0}")]
    AlreadyDeleted(String),
    #[error("Item is not deleted: {0}")]
    NotDeleted(String),
    #[error("Invalid status '{status}'. Allowed: {allowed:?}")]
    InvalidStatus {
        status: String,
        allowed: Vec<String>,
    },
    #[error("Invalid priority {priority}. Must be between 1 and {max}")]
    InvalidPriority { priority: u32, max: u32 },
    #[error("Item already exists: {0}")]
    AlreadyExists(String),
    #[error("Item is deleted: {0}")]
    IsDeleted(String),
    #[error("Organization sync error: {0}")]
    OrgSyncError(String),
    #[error("Cannot move item to same project")]
    SameProject,
    #[error("Target project not initialized")]
    TargetNotInitialized,
    #[error("{0}")]
    Custom(String),
}
#[cfg(test)]
#[path = "../error_tests.rs"]
mod error_tests;
