//! Item metadata types and traits.

use std::collections::HashMap;

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
