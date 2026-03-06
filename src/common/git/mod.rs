//! Git integration utilities.
mod branch;
mod error;
mod git_remote;
pub use branch::is_git_repository;
pub use git_remote::get_remote_origin_url;
#[cfg(test)]
#[path = "../git_tests.rs"]
mod git_tests;
