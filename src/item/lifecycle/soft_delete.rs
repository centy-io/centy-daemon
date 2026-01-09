//! Soft delete and restore traits for items.

use async_trait::async_trait;
use std::path::Path;

use crate::item::core::crud::ItemCrud;
use crate::item::core::error::ItemError;
use crate::item::core::id::ItemId;

/// Trait for items that support soft deletion.
///
/// Soft deletion marks an item as deleted without removing it from storage,
/// allowing for restoration later.
#[async_trait]
pub trait SoftDeletable: ItemCrud {
    /// Result type for soft delete operation
    type SoftDeleteResult;

    /// Soft-delete an item by setting its deleted_at timestamp.
    ///
    /// The item remains in storage but is excluded from normal queries
    /// unless explicitly requested.
    async fn soft_delete(
        project_path: &Path,
        id: &ItemId,
    ) -> Result<Self::SoftDeleteResult, ItemError>;
}

/// Trait for items that can be restored after soft deletion.
#[async_trait]
pub trait Restorable: SoftDeletable {
    /// Result type for restore operation
    type RestoreResult;

    /// Restore a soft-deleted item by clearing its deleted_at timestamp.
    async fn restore(project_path: &Path, id: &ItemId) -> Result<Self::RestoreResult, ItemError>;
}
