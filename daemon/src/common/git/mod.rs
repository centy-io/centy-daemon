//! Git integration utilities.
mod branch;
mod error;
mod git_remote;
#[allow(unused_imports)]
pub use branch::{
    detect_current_branch, get_default_branch, is_git_repository, is_git_root,
    validate_branch_exists,
};
#[allow(unused_imports)]
pub use error::GitError;
pub use git_remote::get_remote_origin_url;
#[allow(unused_imports)]
pub use std::path::Path;
#[cfg(test)]
#[path = "../git_tests.rs"]
mod git_tests;
