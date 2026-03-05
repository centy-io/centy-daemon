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
#[path = "storage_tests_1.rs"]
mod storage_tests_1;
#[cfg(test)]
#[path = "storage_tests_2.rs"]
mod storage_tests_2;
#[cfg(test)]
#[path = "storage_tests_3.rs"]
mod storage_tests_3;
#[cfg(test)]
#[path = "storage_tests_4.rs"]
mod storage_tests_4;
