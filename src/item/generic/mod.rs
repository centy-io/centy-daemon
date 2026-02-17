//! Generic storage layer for config-driven item types.
//!
//! This module provides a type-agnostic CRUD layer that works with any item type
//! defined by an `mdstore::TypeConfig`. It sits alongside the existing entity-specific
//! code (Issue, Doc) without breaking it.

pub mod storage;
pub mod types;

pub use storage::{
    generic_create, generic_delete, generic_duplicate, generic_get, generic_list, generic_move,
    generic_restore, generic_soft_delete, generic_update,
};
pub use types::DuplicateGenericItemOptions;
