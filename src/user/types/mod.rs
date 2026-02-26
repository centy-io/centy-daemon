//! User type definitions and error types.
mod utils;
pub use utils::{slugify, validate_user_id};
use crate::manifest::ManifestError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
/// A project user/team member
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub git_usernames: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
}
/// The users.json file structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsersFile { pub users: Vec<User> }
/// A git contributor found in history
#[derive(Debug, Clone)]
pub struct GitContributor { pub name: String, pub email: String }
/// Result of syncing users from git
#[derive(Debug, Clone, Default)]
pub struct SyncUsersResult {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
    pub errors: Vec<String>,
    pub would_create: Vec<GitContributor>,
    pub would_skip: Vec<GitContributor>,
}
/// User-related errors
#[derive(Error, Debug)]
pub enum UserError {
    #[error("IO error: {0}")] IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")] JsonError(#[from] serde_json::Error),
    #[error("Manifest error: {0}")] ManifestError(#[from] ManifestError),
    #[error("Centy not initialized. Run 'centy init' first.")] NotInitialized,
    #[error("User '{0}' not found")] UserNotFound(String),
    #[error("User '{0}' already exists")] UserAlreadyExists(String),
    #[error("User '{0}' is not soft-deleted")] UserNotDeleted(String),
    #[error("User '{0}' is already soft-deleted")] UserAlreadyDeleted(String),
    #[error("Invalid user ID: {0}")] InvalidUserId(String),
    #[error("Not a git repository")] NotGitRepository,
    #[error("Git command failed: {0}")] GitError(String),
}
#[cfg(test)]
#[path = "../types_tests_1.rs"]
mod types_tests_1;
#[cfg(test)]
#[path = "../types_tests_2.rs"]
mod types_tests_2;
