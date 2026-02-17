//! Daemon-specific generic item types.
//!
//! Core types (Frontmatter, Item, CreateOptions, UpdateOptions, DuplicateResult,
//! MoveResult) now live in mdstore. This module retains daemon-specific types
//! that need project paths rather than type directories.

use std::path::PathBuf;

/// Options for duplicating a generic item (daemon-specific).
///
/// Uses project paths (not type directories) because the daemon needs to
/// resolve the `.centy/<folder>/` path internally.
#[derive(Debug, Clone)]
pub struct DuplicateGenericItemOptions {
    /// Path to the project containing the source item
    pub source_project_path: PathBuf,
    /// Path to the target project (can be same as source)
    pub target_project_path: PathBuf,
    /// ID of the item to duplicate
    pub item_id: String,
    /// Override for the new item's ID (for slug-identified types)
    pub new_id: Option<String>,
    /// Override for the new item's title (default: "Copy of {original}")
    pub new_title: Option<String>,
}
