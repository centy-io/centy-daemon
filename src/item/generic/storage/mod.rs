//! Generic CRUD operations for config-driven item types.
mod crud_ops;
mod helpers;
mod move_ops;
#[cfg(test)]
use crate::item::core::error::ItemError;
#[cfg(test)]
use crate::manifest;
#[cfg(test)]
use crate::utils::get_centy_path;
pub use crud_ops::{
    generic_create, generic_delete, generic_get, generic_get_by_display_number, generic_list,
    generic_restore, generic_soft_delete, generic_update,
};
#[cfg(test)]
use mdstore::{CreateOptions, Filters, TypeConfig, UpdateOptions};
pub use move_ops::{generic_duplicate, generic_move, generic_rename_slug};
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
