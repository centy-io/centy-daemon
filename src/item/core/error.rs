//! Unified error types for item operations.

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

    #[error("Invalid status '{status}'. Allowed: {allowed:?}")]
    InvalidStatus { status: String, allowed: Vec<String> },

    #[error("Invalid priority {priority}. Must be between 1 and {max}")]
    InvalidPriority { priority: u32, max: u32 },

    #[error("Item already exists: {0}")]
    AlreadyExists(String),

    #[error("Item is deleted: {0}")]
    IsDeleted(String),

    #[error("Organization sync error: {0}")]
    OrgSyncError(String),

    #[error("{0}")]
    Custom(String),
}

impl ItemError {
    /// Create a custom error with a message
    pub fn custom(msg: impl Into<String>) -> Self {
        ItemError::Custom(msg.into())
    }

    /// Create a not found error
    pub fn not_found(id: impl Into<String>) -> Self {
        ItemError::NotFound(id.into())
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        ItemError::ValidationError(msg.into())
    }
}
