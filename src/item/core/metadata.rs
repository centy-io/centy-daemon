//! Item metadata types and traits.

use std::collections::HashMap;

// Re-export CommonMetadata from common module during transition
// This will be moved here in a later phase when we break the circular dependency
pub use crate::common::metadata::CommonMetadata;

/// Trait for item metadata providing common timestamp operations.
pub trait ItemMetadata: Sized + Clone + Send + Sync {
    /// Get the creation timestamp
    fn created_at(&self) -> &str;

    /// Get the last update timestamp
    fn updated_at(&self) -> &str;

    /// Get the deletion timestamp if soft-deleted
    fn deleted_at(&self) -> Option<&str>;

    /// Set the update timestamp
    fn set_updated_at(&mut self, timestamp: String);

    /// Set the deletion timestamp (for soft delete)
    fn set_deleted_at(&mut self, timestamp: Option<String>);

    /// Check if this item is soft-deleted
    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }
}

/// Trait for items that have a display number (human-readable sequential ID)
pub trait DisplayNumbered: ItemMetadata {
    /// Get the display number
    fn display_number(&self) -> u32;

    /// Set the display number
    fn set_display_number(&mut self, num: u32);
}

/// Trait for items that have a status field
pub trait Statusable: ItemMetadata {
    /// Get the current status
    fn status(&self) -> &str;

    /// Set the status
    fn set_status(&mut self, status: String);
}

/// Trait for items that have a priority field
pub trait Prioritized: ItemMetadata {
    /// Get the priority (1 = highest)
    fn priority(&self) -> u32;

    /// Set the priority
    fn set_priority(&mut self, priority: u32);
}

/// Trait for items that support custom fields
pub trait CustomFielded: ItemMetadata {
    /// Get custom fields
    fn custom_fields(&self) -> &HashMap<String, serde_json::Value>;

    /// Get mutable reference to custom fields
    fn custom_fields_mut(&mut self) -> &mut HashMap<String, serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_metadata_new() {
        let metadata = CommonMetadata::new(1, "open".to_string(), 1, HashMap::new());
        assert_eq!(metadata.display_number, 1);
        assert_eq!(metadata.status, "open");
        assert_eq!(metadata.priority, 1);
        assert!(!metadata.created_at.is_empty());
        assert!(!metadata.updated_at.is_empty());
    }

    #[test]
    fn test_deserialize_priority_number() {
        let json =
            r#"{"status":"open","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: CommonMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.priority, 1);
    }

    #[test]
    fn test_deserialize_priority_string_high() {
        let json = r#"{"status":"open","priority":"high","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: CommonMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.priority, 1);
    }

    #[test]
    fn test_deserialize_priority_string_medium() {
        let json = r#"{"status":"open","priority":"medium","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: CommonMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.priority, 2);
    }

    #[test]
    fn test_deserialize_priority_string_low() {
        let json = r#"{"status":"open","priority":"low","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: CommonMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.priority, 3);
    }

    #[test]
    fn test_serialize_priority_as_number() {
        let metadata = CommonMetadata::new(1, "open".to_string(), 2, HashMap::new());
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""priority":2"#));
    }

    #[test]
    fn test_deserialize_legacy_without_display_number() {
        // Legacy entities without display_number should default to 0
        let json =
            r#"{"status":"open","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: CommonMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.display_number, 0);
    }

    #[test]
    fn test_serialize_display_number() {
        let metadata = CommonMetadata::new(42, "open".to_string(), 1, HashMap::new());
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""displayNumber":42"#));
    }
}
