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

impl From<mdstore::StoreError> for ItemError {
    fn from(err: mdstore::StoreError) -> Self {
        match err {
            mdstore::StoreError::IoError(e) => ItemError::IoError(e),
            mdstore::StoreError::NotFound(id) => ItemError::NotFound(id),
            mdstore::StoreError::ValidationError(msg) => ItemError::ValidationError(msg),
            mdstore::StoreError::JsonError(e) => ItemError::JsonError(e),
            mdstore::StoreError::YamlError(msg) => ItemError::YamlError(msg),
            mdstore::StoreError::FrontmatterError(msg) => ItemError::FrontmatterError(msg),
            mdstore::StoreError::ItemTypeNotFound(msg) => ItemError::ItemTypeNotFound(msg),
            mdstore::StoreError::FeatureNotEnabled(msg) => ItemError::FeatureNotEnabled(msg),
            mdstore::StoreError::AlreadyDeleted(id) => ItemError::AlreadyDeleted(id),
            mdstore::StoreError::NotDeleted(id) => ItemError::NotDeleted(id),
            mdstore::StoreError::InvalidStatus { status, allowed } => {
                ItemError::InvalidStatus { status, allowed }
            }
            mdstore::StoreError::InvalidPriority { priority, max } => {
                ItemError::InvalidPriority { priority, max }
            }
            mdstore::StoreError::AlreadyExists(id) => ItemError::AlreadyExists(id),
            mdstore::StoreError::IsDeleted(id) => ItemError::IsDeleted(id),
            mdstore::StoreError::SameLocation => ItemError::SameProject,
            mdstore::StoreError::Custom(msg) => ItemError::Custom(msg),
        }
    }
}

impl From<mdstore::ConfigError> for ItemError {
    fn from(err: mdstore::ConfigError) -> Self {
        match err {
            mdstore::ConfigError::IoError(e) => ItemError::IoError(e),
            mdstore::ConfigError::YamlError(e) => ItemError::YamlError(e.to_string()),
            mdstore::ConfigError::JsonError(e) => ItemError::JsonError(e),
        }
    }
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
