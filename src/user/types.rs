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
    fn test_slugify_special_chars() {
        // cspell:ignore malley
        assert_eq!(slugify("O'Malley"), "o-malley");
        assert_eq!(slugify("user@email.com"), "user-email-com");
        assert_eq!(slugify("first.last"), "first-last");
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

    #[test]
    fn test_validate_user_id_error_messages() {
        let err = validate_user_id("").unwrap_err();
        assert!(format!("{err}").contains("empty"));

        let err = validate_user_id("UPPER").unwrap_err();
        assert!(format!("{err}").contains("lowercase"));

        let err = validate_user_id("-start").unwrap_err();
        assert!(format!("{err}").contains("hyphen"));
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: "john-doe".to_string(),
            name: "John Doe".to_string(),
            email: Some("john@example.com".to_string()),
            git_usernames: vec!["johndoe".to_string()],
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-06-15T12:00:00Z".to_string(),
            deleted_at: None,
        };

        let json = serde_json::to_string(&user).expect("Should serialize");
        let deserialized: User = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.id, "john-doe");
        assert_eq!(deserialized.name, "John Doe");
        assert_eq!(deserialized.email, Some("john@example.com".to_string()));
        assert_eq!(deserialized.git_usernames, vec!["johndoe"]);
    }

    #[test]
    fn test_user_serialization_camel_case() {
        let user = User {
            id: "test".to_string(),
            name: "Test".to_string(),
            email: None,
            git_usernames: vec!["gh-user".to_string()],
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
            deleted_at: None,
        };

        let json = serde_json::to_string(&user).expect("Should serialize");
        assert!(json.contains("createdAt"));
        assert!(json.contains("updatedAt"));
        assert!(json.contains("gitUsernames"));
        assert!(!json.contains("created_at"));
        assert!(!json.contains("git_usernames"));
    }

    #[test]
    fn test_user_skip_serializing_empty_fields() {
        let user = User {
            id: "test".to_string(),
            name: "Test".to_string(),
            email: None,
            git_usernames: vec![],
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
            deleted_at: None,
        };

        let json = serde_json::to_string(&user).expect("Should serialize");
        assert!(!json.contains("email"));
        assert!(!json.contains("gitUsernames"));
        assert!(!json.contains("deletedAt"));
    }

    #[test]
    fn test_user_with_deleted_at() {
        let user = User {
            id: "test".to_string(),
            name: "Test".to_string(),
            email: None,
            git_usernames: vec![],
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
            deleted_at: Some("2024-06-15T12:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&user).expect("Should serialize");
        assert!(json.contains("deletedAt"));
    }

    #[test]
    fn test_users_file_serialization() {
        let users_file = UsersFile {
            users: vec![User {
                id: "alice".to_string(),
                name: "Alice".to_string(),
                email: None,
                git_usernames: vec![],
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
                deleted_at: None,
            }],
        };

        let json = serde_json::to_string(&users_file).expect("Should serialize");
        let deserialized: UsersFile = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.users.len(), 1);
        assert_eq!(deserialized.users[0].id, "alice");
    }

    #[test]
    fn test_users_file_default() {
        let users_file = UsersFile::default();
        assert!(users_file.users.is_empty());
    }

    #[test]
    fn test_git_contributor_debug() {
        let contributor = GitContributor {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        let debug = format!("{contributor:?}");
        assert!(debug.contains("Alice"));
        assert!(debug.contains("alice@example.com"));
    }

    #[test]
    fn test_sync_users_result_default() {
        let result = SyncUsersResult::default();
        assert!(result.created.is_empty());
        assert!(result.skipped.is_empty());
        assert!(result.errors.is_empty());
        assert!(result.would_create.is_empty());
        assert!(result.would_skip.is_empty());
    }

    #[test]
    fn test_user_error_display() {
        assert_eq!(
            format!("{}", UserError::NotInitialized),
            "Centy not initialized. Run 'centy init' first."
        );
        assert_eq!(
            format!("{}", UserError::UserNotFound("john".to_string())),
            "User 'john' not found"
        );
        assert_eq!(
            format!("{}", UserError::UserAlreadyExists("john".to_string())),
            "User 'john' already exists"
        );
        assert_eq!(
            format!("{}", UserError::UserNotDeleted("john".to_string())),
            "User 'john' is not soft-deleted"
        );
        assert_eq!(
            format!("{}", UserError::UserAlreadyDeleted("john".to_string())),
            "User 'john' is already soft-deleted"
        );
        assert_eq!(
            format!("{}", UserError::NotGitRepository),
            "Not a git repository"
        );
    }

    #[test]
    fn test_user_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err = UserError::from(io_err);
        assert!(matches!(err, UserError::IoError(_)));
    }
}
