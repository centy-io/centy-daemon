//! Move operation trait for items.

use async_trait::async_trait;

use crate::item::core::crud::ItemCrud;
use crate::item::core::error::ItemError;

/// Trait for items that can be moved between projects.
#[async_trait]
pub trait Movable: ItemCrud {
    /// Options for moving an item
    type MoveOptions;
    /// Result of moving an item
    type MoveResult;

    /// Move an item from one project to another.
    ///
    /// This operation:
    /// 1. Creates the item in the target project
    /// 2. Deletes the item from the source project
    /// 3. Updates any cross-references
    async fn move_item(options: Self::MoveOptions) -> Result<Self::MoveResult, ItemError>;
}
