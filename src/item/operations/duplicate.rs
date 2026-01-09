//! Duplicate operation trait for items.

use async_trait::async_trait;

use crate::item::core::crud::ItemCrud;
use crate::item::core::error::ItemError;

/// Trait for items that can be duplicated.
#[async_trait]
pub trait Duplicable: ItemCrud {
    /// Options for duplicating an item
    type DuplicateOptions;
    /// Result of duplicating an item
    type DuplicateResult;

    /// Create a copy of an item, optionally in a different project.
    ///
    /// The duplicated item gets a new ID and display number but preserves
    /// the content and metadata from the original.
    async fn duplicate(options: Self::DuplicateOptions) -> Result<Self::DuplicateResult, ItemError>;
}
