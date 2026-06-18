//! Item metadata types and traits.

use std::collections::HashMap;

/// Trait for item metadata providing common timestamp operations.
pub trait ItemMetadata: Sized + Clone + Send + Sync {
    /// Get the creation timestamp
    fn created_at(&self) -> &str;

    /// Get the deletion timestamp if soft-deleted
    fn deleted_at(&self) -> Option<&str>;

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
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::struct_field_names)]
    use super::*;

    #[derive(Clone)]
    struct MockItem {
        created_at: String,
        deleted_at: Option<String>,
    }

    impl ItemMetadata for MockItem {
        fn created_at(&self) -> &str {
            &self.created_at
        }
        fn deleted_at(&self) -> Option<&str> {
            self.deleted_at.as_deref()
        }
        fn set_deleted_at(&mut self, timestamp: Option<String>) {
            self.deleted_at = timestamp;
        }
    }

    #[test]
    fn test_is_deleted_returns_false_when_no_deleted_at() {
        let item = MockItem {
            created_at: "2024-01-01".to_string(),
            deleted_at: None,
        };
        assert!(!item.is_deleted());
    }

    #[test]
    fn test_is_deleted_returns_true_when_deleted_at_set() {
        let item = MockItem {
            created_at: "2024-01-01".to_string(),
            deleted_at: Some("2024-06-01T00:00:00Z".to_string()),
        };
        assert!(item.is_deleted());
    }
}
