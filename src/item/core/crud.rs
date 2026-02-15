//! Generic CRUD operations trait for items.

use async_trait::async_trait;
use std::path::Path;

use super::error::ItemError;
use super::id::{Identifiable, ItemId};
use super::metadata::ItemMetadata;

/// Common filter options for listing items
#[derive(Debug, Clone, Default)]
pub struct ItemFilters {
    /// Filter by status
    pub status: Option<String>,
    /// Filter by priority
    pub priority: Option<u32>,
    /// Include soft-deleted items
    pub include_deleted: bool,
    /// Limit number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

impl ItemFilters {
    /// Create a new empty filter
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by status
    #[must_use]
    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Filter by priority
    #[must_use]
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Include soft-deleted items
    #[must_use]
    pub fn include_deleted(mut self) -> Self {
        self.include_deleted = true;
        self
    }

    /// Limit results
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Offset results for pagination
    #[must_use]
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Core Item trait that all entities (Issue, Doc) must implement.
///
/// This trait defines the fundamental properties that every item in the system shares.
pub trait Item: Identifiable + Send + Sync + Sized {
    /// The metadata type associated with this item
    type Metadata: ItemMetadata;

    /// The storage folder name (e.g., "issues", "docs")
    const STORAGE_FOLDER: &'static str;

    /// Get the item's title
    fn title(&self) -> &str;

    /// Get the item's description/content
    fn description(&self) -> &str;

    /// Get the item's metadata
    fn metadata(&self) -> &Self::Metadata;

    /// Get mutable reference to the item's metadata
    fn metadata_mut(&mut self) -> &mut Self::Metadata;

    /// Get the content file name for this item
    fn content_filename(&self) -> String;

    /// Get the metadata file name for this item (None if metadata is embedded)
    fn metadata_filename(&self) -> Option<String>;
}

/// Generic result wrapper for items with project context
#[derive(Debug, Clone)]
pub struct ItemWithProject<T: Item> {
    /// The item itself
    pub item: T,
    /// Path of the project containing this item
    pub project_path: String,
    /// Name of the project
    pub project_name: String,
}

/// Generic CRUD operations for items.
///
/// This trait defines the standard create, read, update, delete operations
/// that all item types should support.
#[async_trait]
pub trait ItemCrud: Item {
    /// Options for creating a new item
    type CreateOptions;
    /// Result of creating a new item
    type CreateResult;
    /// Options for updating an existing item
    type UpdateOptions;
    /// Result of updating an item
    type UpdateResult;

    /// Create a new item
    async fn create(
        project_path: &Path,
        options: Self::CreateOptions,
    ) -> Result<Self::CreateResult, ItemError>;

    /// Get an item by its identifier
    async fn get(project_path: &Path, id: &ItemId) -> Result<Self, ItemError>;

    /// List items with optional filters
    async fn list(project_path: &Path, filters: ItemFilters) -> Result<Vec<Self>, ItemError>;

    /// Update an existing item
    async fn update(
        project_path: &Path,
        id: &ItemId,
        options: Self::UpdateOptions,
    ) -> Result<Self::UpdateResult, ItemError>;

    /// Delete an item (hard delete)
    async fn delete(project_path: &Path, id: &ItemId) -> Result<(), ItemError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_filters_default() {
        let filters = ItemFilters::default();
        assert!(filters.status.is_none());
        assert!(filters.priority.is_none());
        assert!(!filters.include_deleted);
        assert!(filters.limit.is_none());
        assert!(filters.offset.is_none());
    }

    #[test]
    fn test_item_filters_new() {
        let filters = ItemFilters::new();
        assert!(filters.status.is_none());
        assert!(filters.priority.is_none());
        assert!(!filters.include_deleted);
        assert!(filters.limit.is_none());
        assert!(filters.offset.is_none());
    }

    #[test]
    fn test_item_filters_with_status() {
        let filters = ItemFilters::new().with_status("open");
        assert_eq!(filters.status, Some("open".to_string()));
    }

    #[test]
    fn test_item_filters_with_status_string() {
        let filters = ItemFilters::new().with_status("in-progress".to_string());
        assert_eq!(filters.status, Some("in-progress".to_string()));
    }

    #[test]
    fn test_item_filters_with_priority() {
        let filters = ItemFilters::new().with_priority(1);
        assert_eq!(filters.priority, Some(1));
    }

    #[test]
    fn test_item_filters_include_deleted() {
        let filters = ItemFilters::new().include_deleted();
        assert!(filters.include_deleted);
    }

    #[test]
    fn test_item_filters_with_limit() {
        let filters = ItemFilters::new().with_limit(10);
        assert_eq!(filters.limit, Some(10));
    }

    #[test]
    fn test_item_filters_with_offset() {
        let filters = ItemFilters::new().with_offset(5);
        assert_eq!(filters.offset, Some(5));
    }

    #[test]
    fn test_item_filters_chained() {
        let filters = ItemFilters::new()
            .with_status("open")
            .with_priority(2)
            .include_deleted()
            .with_limit(20)
            .with_offset(10);

        assert_eq!(filters.status, Some("open".to_string()));
        assert_eq!(filters.priority, Some(2));
        assert!(filters.include_deleted);
        assert_eq!(filters.limit, Some(20));
        assert_eq!(filters.offset, Some(10));
    }

    #[test]
    fn test_item_filters_clone() {
        let filters = ItemFilters::new().with_status("open").with_priority(1);
        let cloned = filters.clone();
        assert_eq!(cloned.status, Some("open".to_string()));
        assert_eq!(cloned.priority, Some(1));
    }

    #[test]
    fn test_item_filters_debug() {
        let filters = ItemFilters::new().with_status("open");
        let debug = format!("{filters:?}");
        assert!(debug.contains("ItemFilters"));
        assert!(debug.contains("open"));
    }
}
