use super::*;
#[test]
fn test_user_with_deleted_at() {
    let user = User {
        id: "test".to_string(), name: "Test".to_string(), email: None, git_usernames: vec![],
        created_at: "2024-01-01".to_string(), updated_at: "2024-01-01".to_string(),
        deleted_at: Some("2024-06-15T12:00:00Z".to_string()),
    };
    let json = serde_json::to_string(&user).expect("Should serialize");
    assert!(json.contains("deletedAt"));
}
#[test]
fn test_users_file_serialization() {
    let users_file = UsersFile { users: vec![User {
        id: "alice".to_string(), name: "Alice".to_string(), email: None, git_usernames: vec![],
        created_at: "2024-01-01".to_string(), updated_at: "2024-01-01".to_string(),
        deleted_at: None,
    }]};
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
    let contributor = GitContributor { name: "Alice".to_string(), email: "alice@example.com".to_string() };
    let debug = format!("{contributor:?}");
    assert!(debug.contains("Alice")); assert!(debug.contains("alice@example.com"));
}
#[test]
fn test_sync_users_result_default() {
    let result = SyncUsersResult::default();
    assert!(result.created.is_empty()); assert!(result.skipped.is_empty());
    assert!(result.errors.is_empty()); assert!(result.would_create.is_empty());
    assert!(result.would_skip.is_empty());
}
#[test]
fn test_user_error_display() {
    assert_eq!(format!("{}", UserError::NotInitialized),
        "Centy not initialized. Run 'centy init' first.");
    assert_eq!(format!("{}", UserError::UserNotFound("john".to_string())),
        "User 'john' not found");
    assert_eq!(format!("{}", UserError::UserAlreadyExists("john".to_string())),
        "User 'john' already exists");
    assert_eq!(format!("{}", UserError::UserNotDeleted("john".to_string())),
        "User 'john' is not soft-deleted");
    assert_eq!(format!("{}", UserError::UserAlreadyDeleted("john".to_string())),
        "User 'john' is already soft-deleted");
    assert_eq!(format!("{}", UserError::NotGitRepository), "Not a git repository");
}
#[test]
fn test_user_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err = UserError::from(io_err);
    assert!(matches!(err, UserError::IoError(_)));
}
