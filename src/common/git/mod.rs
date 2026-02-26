//! Git integration utilities.
#![allow(dead_code)]
mod error;
mod branch;
mod git_remote;
mod worktree;
#[allow(unused_imports)]
pub use error::GitError;
#[allow(unused_imports)]
pub use branch::{detect_current_branch, get_default_branch, is_git_repository, validate_branch_exists};
#[allow(unused_imports)]
pub use std::path::Path;
pub use git_remote::get_remote_origin_url;
#[allow(unused_imports)]
pub use worktree::{create_worktree, prune_worktrees, remove_worktree};
#[cfg(test)]
#[path = "../git_tests.rs"]
mod git_tests;
