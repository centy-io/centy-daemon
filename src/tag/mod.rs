//! Tag management module.
//!
//! This module provides functionality for managing project tags:
//! - Creating, reading, updating, and deleting tags
//! - Assigning tags to entities (issues, docs, PRs)
//! - Organization-level tag synchronization
//!
//! Tags are stored in `.centy/tags.json` in each project.

mod crud;
mod storage;
mod types;

#[allow(unused_imports)]
pub use crud::{
    create_tag, delete_tag, get_tag, list_tags, update_tag,
    CreateTagOptions, CreateTagResult, DeleteTagResult, UpdateTagOptions, UpdateTagResult,
};
#[allow(unused_imports)]
pub use storage::{find_tag_by_name, find_tag_index_by_name, read_tags, write_tags};
#[allow(unused_imports)]
pub use types::{slugify_tag_name, validate_color, validate_tag_name, Tag, TagError, TagsFile};
