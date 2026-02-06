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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_error_custom() {
        let err = ItemError::custom("something went wrong");
        assert!(matches!(err, ItemError::Custom(_)));
        assert_eq!(format!("{err}"), "something went wrong");
    }

    #[test]
    fn test_item_error_not_found() {
        let err = ItemError::not_found("issue-123");
        assert!(matches!(err, ItemError::NotFound(_)));
        assert_eq!(format!("{err}"), "Item not found: issue-123");
    }

    #[test]
    fn test_item_error_validation() {
        let err = ItemError::validation("title is required");
        assert!(matches!(err, ItemError::ValidationError(_)));
        assert_eq!(format!("{err}"), "Validation error: title is required");
    }

    #[test]
    fn test_item_error_not_initialized() {
        let err = ItemError::NotInitialized;
        assert_eq!(format!("{err}"), "Project not initialized");
    }

    #[test]
    fn test_item_error_invalid_status() {
        let err = ItemError::InvalidStatus {
            status: "invalid".to_string(),
            allowed: vec!["open".to_string(), "closed".to_string()],
        };
        let display = format!("{err}");
        assert!(display.contains("Invalid status 'invalid'"));
        assert!(display.contains("open"));
        assert!(display.contains("closed"));
    }

    #[test]
    fn test_item_error_invalid_priority() {
        let err = ItemError::InvalidPriority {
            priority: 10,
            max: 3,
        };
        let display = format!("{err}");
        assert!(display.contains("Invalid priority 10"));
        assert!(display.contains("between 1 and 3"));
    }

    #[test]
    fn test_item_error_already_exists() {
        let err = ItemError::AlreadyExists("abc-123".to_string());
        assert_eq!(format!("{err}"), "Item already exists: abc-123");
    }

    #[test]
    fn test_item_error_is_deleted() {
        let err = ItemError::IsDeleted("abc-123".to_string());
        assert_eq!(format!("{err}"), "Item is deleted: abc-123");
    }

    #[test]
    fn test_item_error_org_sync_error() {
        let err = ItemError::OrgSyncError("sync failed".to_string());
        assert_eq!(format!("{err}"), "Organization sync error: sync failed");
    }

    #[test]
    fn test_item_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ItemError::from(io_err);
        assert!(matches!(err, ItemError::IoError(_)));
        let display = format!("{err}");
        assert!(display.contains("IO error"));
    }

    #[test]
    fn test_item_error_custom_with_string() {
        let err = ItemError::custom(String::from("dynamic error"));
        assert_eq!(format!("{err}"), "dynamic error");
    }

    #[test]
    fn test_item_error_not_found_with_string() {
        let err = ItemError::not_found(String::from("doc-xyz"));
        assert_eq!(format!("{err}"), "Item not found: doc-xyz");
    }

    #[test]
    fn test_item_error_validation_with_string() {
        let err = ItemError::validation(String::from("bad input"));
        assert_eq!(format!("{err}"), "Validation error: bad input");
    }
}
