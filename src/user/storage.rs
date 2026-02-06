//! User storage operations for reading/writing users.json.

use super::types::{User, UserError, UsersFile};
use crate::manifest::read_manifest;
use crate::utils::get_centy_path;
use std::path::Path;
use tokio::fs;

/// Read users from the .centy/users.json file.
/// Returns an empty list if the file doesn't exist.
pub async fn read_users(project_path: &Path) -> Result<Vec<User>, UserError> {
    // Verify project is initialized
    read_manifest(project_path)
        .await
        .map_err(|_| UserError::NotInitialized)?
        .ok_or(UserError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let users_path = centy_path.join("users.json");

    if !users_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&users_path).await?;
    let users_file: UsersFile = serde_json::from_str(&content)?;

    Ok(users_file.users)
}

/// Write users to the .centy/users.json file.
pub async fn write_users(project_path: &Path, users: &[User]) -> Result<(), UserError> {
    let centy_path = get_centy_path(project_path);
    let users_path = centy_path.join("users.json");

    let users_file = UsersFile {
        users: users.to_vec(),
    };

    let content = serde_json::to_string_pretty(&users_file)?;
    fs::write(&users_path, content).await?;

    Ok(())
}

/// Check if a user with the given email already exists.
pub fn find_user_by_email<'a>(users: &'a [User], email: &str) -> Option<&'a User> {
    users.iter().find(|u| u.email.as_deref() == Some(email))
}

/// Check if a user with the given ID already exists.
pub fn find_user_by_id<'a>(users: &'a [User], id: &str) -> Option<&'a User> {
    users.iter().find(|u| u.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_user(id: &str, name: &str, email: Option<&str>) -> User {
        User {
            id: id.to_string(),
            name: name.to_string(),
            email: email.map(ToString::to_string),
            git_usernames: vec![],
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            deleted_at: None,
        }
    }

    #[test]
    fn test_find_user_by_email_found() {
        let users = vec![
            make_user("alice", "Alice", Some("alice@example.com")),
            make_user("bob", "Bob", Some("bob@example.com")),
        ];

        let found = find_user_by_email(&users, "alice@example.com");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "alice");
    }

    #[test]
    fn test_find_user_by_email_not_found() {
        let users = vec![make_user("alice", "Alice", Some("alice@example.com"))];

        let found = find_user_by_email(&users, "unknown@example.com");
        assert!(found.is_none());
    }

    #[test]
    fn test_find_user_by_email_none_emails() {
        let users = vec![make_user("alice", "Alice", None)];

        let found = find_user_by_email(&users, "alice@example.com");
        assert!(found.is_none());
    }

    #[test]
    fn test_find_user_by_email_empty_list() {
        let users: Vec<User> = vec![];
        let found = find_user_by_email(&users, "test@example.com");
        assert!(found.is_none());
    }

    #[test]
    fn test_find_user_by_id_found() {
        let users = vec![
            make_user("alice", "Alice", None),
            make_user("bob", "Bob", None),
        ];

        let found = find_user_by_id(&users, "bob");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Bob");
    }

    #[test]
    fn test_find_user_by_id_not_found() {
        let users = vec![make_user("alice", "Alice", None)];

        let found = find_user_by_id(&users, "unknown");
        assert!(found.is_none());
    }

    #[test]
    fn test_find_user_by_id_empty_list() {
        let users: Vec<User> = vec![];
        let found = find_user_by_id(&users, "test");
        assert!(found.is_none());
    }
}
