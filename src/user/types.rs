//! User type definitions and error types.

use crate::manifest::ManifestError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A project user/team member
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// Unique identifier (slug format, e.g., "john-doe")
    pub id: String,
    /// Display name
    pub name: String,
    /// Email address (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Git usernames (e.g., GitHub handles)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub git_usernames: Vec<String>,
    /// ISO timestamp when created
    pub created_at: String,
    /// ISO timestamp when last updated
    pub updated_at: String,
    /// ISO timestamp when soft-deleted (None if not deleted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
}

/// The users.json file structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsersFile {
    pub users: Vec<User>,
}

/// A git contributor found in history
#[derive(Debug, Clone)]
pub struct GitContributor {
    pub name: String,
    pub email: String,
}

/// Result of syncing users from git
#[derive(Debug, Clone, Default)]
pub struct SyncUsersResult {
    /// User IDs that were created
    pub created: Vec<String>,
    /// Emails that were skipped (already exist)
    pub skipped: Vec<String>,
    /// Errors during creation
    pub errors: Vec<String>,
    /// For dry run: users that would be created
    pub would_create: Vec<GitContributor>,
    /// For dry run: users that would be skipped
    pub would_skip: Vec<GitContributor>,
}

/// User-related errors
#[derive(Error, Debug)]
pub enum UserError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] ManifestError),

    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,

    #[error("User '{0}' not found")]
    UserNotFound(String),

    #[error("User '{0}' already exists")]
    UserAlreadyExists(String),

    #[error("User '{0}' is not soft-deleted")]
    UserNotDeleted(String),

    #[error("User '{0}' is already soft-deleted")]
    UserAlreadyDeleted(String),

    #[error("Invalid user ID: {0}")]
    InvalidUserId(String),

    #[error("Not a git repository")]
    NotGitRepository,

    #[error("Git command failed: {0}")]
    GitError(String),
}

/// Convert a name to a URL-friendly slug (kebab-case)
pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Validate a user ID (must be non-empty, lowercase alphanumeric with hyphens)
pub fn validate_user_id(id: &str) -> Result<(), UserError> {
    if id.is_empty() {
        return Err(UserError::InvalidUserId("ID cannot be empty".to_string()));
    }

    if !id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(UserError::InvalidUserId(
            "ID must contain only lowercase letters, numbers, and hyphens".to_string(),
        ));
    }

    if id.starts_with('-') || id.ends_with('-') {
        return Err(UserError::InvalidUserId(
            "ID cannot start or end with a hyphen".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("John Doe"), "john-doe");
        assert_eq!(slugify("Jane Smith"), "jane-smith");
        assert_eq!(slugify("Bob"), "bob");
        assert_eq!(slugify("Test  User"), "test-user");
        assert_eq!(slugify("  leading"), "leading");
        assert_eq!(slugify("trailing  "), "trailing");
        assert_eq!(slugify("UPPERCASE NAME"), "uppercase-name");
        assert_eq!(slugify("user123"), "user123");
    }

    #[test]
    fn test_validate_user_id() {
        assert!(validate_user_id("john-doe").is_ok());
        assert!(validate_user_id("jane-smith-123").is_ok());
        assert!(validate_user_id("bob").is_ok());

        assert!(validate_user_id("").is_err());
        assert!(validate_user_id("-start-with-hyphen").is_err());
        assert!(validate_user_id("end-with-hyphen-").is_err());
        assert!(validate_user_id("UPPERCASE").is_err());
        assert!(validate_user_id("has spaces").is_err());
        assert!(validate_user_id("has_underscore").is_err());
    }
}
