//! Generic CRUD operations for config-driven item types.
mod crud_ops;
mod crud_search;
mod helpers;
mod move_item;
mod move_ops;
#[cfg(test)]
use crate::item::core::error::ItemError;
#[cfg(test)]
use crate::manifest;
pub use crud_ops::{
    generic_create, generic_delete, generic_get, generic_list, generic_restore,
    generic_soft_delete, generic_update,
};
pub use crud_search::generic_get_by_display_number;
#[cfg(test)]
use mdstore::{CreateOptions, Filters, TypeConfig, UpdateOptions};
pub use move_item::generic_move;
pub use move_ops::{generic_duplicate, generic_rename_slug};
#[cfg(test)]
use tokio::fs;
#[cfg(test)]
#[path = "create_and_get.rs"]
mod create_and_get;
#[cfg(test)]
#[path = "deletion_constraints.rs"]
mod deletion_constraints;
#[cfg(test)]
#[path = "helpers_tests.rs"]
mod helpers_tests;
#[cfg(test)]
#[path = "move_ops_tests.rs"]
mod move_ops_tests;
#[cfg(test)]
#[path = "priority_validation.rs"]
mod priority_validation;
#[cfg(test)]
#[path = "soft_delete.rs"]
mod soft_delete;
