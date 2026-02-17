//! Generic CRUD operations trait for items.

use async_trait::async_trait;
use std::path::Path;

use super::error::ItemError;
use super::metadata::ItemMetadata;
use mdstore::{Identifiable, ItemId};

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
    async fn list(project_path: &Path, filters: mdstore::Filters) -> Result<Vec<Self>, ItemError>;

    /// Update an existing item
    async fn update(
        project_path: &Path,
        id: &ItemId,
        options: Self::UpdateOptions,
    ) -> Result<Self::UpdateResult, ItemError>;

    /// Delete an item (hard delete)
    async fn delete(project_path: &Path, id: &ItemId) -> Result<(), ItemError>;
}
