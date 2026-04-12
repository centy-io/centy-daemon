#![allow(clippy::indexing_slicing)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use centy_daemon::user::{
    create_user, delete_user, get_user, list_users, read_users, restore_user, soft_delete_user,
    update_user, write_users, CreateUserOptions, UpdateUserOptions, User, UserError,
};
use common::{create_test_dir, init_centy_project};

fn make_user(id: &str, name: &str) -> User {
    User {
        id: id.to_string(),
        name: name.to_string(),
        email: None,
        git_usernames: vec![],
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        deleted_at: None,
    }
}

fn assert_user_not_found(result: Result<impl std::fmt::Debug, UserError>, expected_id: &str) {
    match result {
        Err(UserError::UserNotFound(id)) => {
            assert_eq!(id, expected_id, "UserNotFound ID mismatch");
        }
        other => panic!("Expected UserNotFound({expected_id}), got: {other:?}"),
    }
}

fn assert_not_initialized<T>(result: &Result<T, UserError>) {
    assert!(
        matches!(result, Err(UserError::NotInitialized)),
        "Expected NotInitialized"
    );
}

// ---------------------------------------------------------------------------
// read_users / write_users (storage layer)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_read_users_not_initialized() {
    let dir = create_test_dir();
    let project_path = dir.path();

    // No .centy directory → NotInitialized
    let result = read_users(project_path).await;
    assert!(
        matches!(result, Err(UserError::NotInitialized)),
        "Expected NotInitialized, got: {result:?}"
    );
}

#[tokio::test]
async fn test_read_users_initialized_no_file() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    // Initialized but users.json does not exist yet → empty vec
    let users = read_users(project_path)
        .await
        .expect("Should return empty vec");
    assert!(users.is_empty(), "Expected no users before any are created");
}

#[tokio::test]
async fn test_write_then_read_users_roundtrip() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    let original = vec![make_user("alice", "Alice"), make_user("bob", "Bob")];
    write_users(project_path, &original)
        .await
        .expect("Should write");

    let loaded = read_users(project_path).await.expect("Should read");
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].id, "alice");
    assert_eq!(loaded[1].id, "bob");
}

// ---------------------------------------------------------------------------
// create_user
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_user_explicit_id() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    let result = create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice Smith".to_string(),
            email: Some("alice@example.com".to_string()),
            git_usernames: vec!["alice-git".to_string()],
        },
    )
    .await
    .expect("Should create user");

    assert_eq!(result.user.id, "alice");
    assert_eq!(result.user.name, "Alice Smith");
    assert_eq!(result.user.email.as_deref(), Some("alice@example.com"));
    assert_eq!(result.user.git_usernames, vec!["alice-git"]);
    assert!(result.user.deleted_at.is_none());
}

#[tokio::test]
async fn test_create_user_auto_slugify_id() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    // Empty id → slugified from name
    let result = create_user(
        project_path,
        CreateUserOptions {
            id: String::new(),
            name: "Jane Doe".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("Should create user with auto-slug");

    assert_eq!(result.user.id, "jane-doe");
}

#[tokio::test]
async fn test_create_user_duplicate_id_error() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    let opts = || CreateUserOptions {
        id: "bob".to_string(),
        name: "Bob".to_string(),
        email: None,
        git_usernames: vec![],
    };

    create_user(project_path, opts())
        .await
        .expect("First create should succeed");

    let second = create_user(project_path, opts()).await;
    match second {
        Err(UserError::UserAlreadyExists(id)) => {
            assert_eq!(id, "bob");
        }
        Err(e) => panic!("Expected UserAlreadyExists(bob), got Err: {e}"),
        Ok(_) => panic!("Expected UserAlreadyExists(bob), got Ok"),
    }
}

#[tokio::test]
async fn test_create_user_not_initialized() {
    let dir = create_test_dir();
    let project_path = dir.path();

    let result = create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await;

    assert_not_initialized(&result.map(|r| r.user));
}

#[tokio::test]
async fn test_create_user_invalid_id() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    // ID with uppercase letters is invalid
    let result = create_user(
        project_path,
        CreateUserOptions {
            id: "Alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await;

    assert!(
        matches!(result, Err(UserError::InvalidUserId(_))),
        "Expected InvalidUserId for uppercase ID"
    );
}

#[tokio::test]
async fn test_create_user_sorted_after_create() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    // Create in reverse alphabetical order
    create_user(
        project_path,
        CreateUserOptions {
            id: "zara".to_string(),
            name: "Zara".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create zara");

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create alice");

    let users = read_users(project_path).await.expect("read");
    assert_eq!(users[0].id, "alice");
    assert_eq!(users[1].id, "zara");
}

// ---------------------------------------------------------------------------
// get_user
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_user_found() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create");

    let user = get_user(project_path, "alice")
        .await
        .expect("Should find alice");
    assert_eq!(user.id, "alice");
}

#[tokio::test]
async fn test_get_user_not_found() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    let result = get_user(project_path, "nonexistent").await;
    assert_user_not_found(result, "nonexistent");
}

// ---------------------------------------------------------------------------
// list_users
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_users_empty() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    let users = list_users(project_path, None, false)
        .await
        .expect("Should list");
    assert!(users.is_empty());
}

#[tokio::test]
async fn test_list_users_excludes_deleted_by_default() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create alice");

    create_user(
        project_path,
        CreateUserOptions {
            id: "bob".to_string(),
            name: "Bob".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create bob");

    // Soft-delete bob
    soft_delete_user(project_path, "bob")
        .await
        .expect("soft-delete bob");

    // Without include_deleted → only alice
    let users = list_users(project_path, None, false).await.expect("list");
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, "alice");
}

#[tokio::test]
async fn test_list_users_include_deleted() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create alice");

    create_user(
        project_path,
        CreateUserOptions {
            id: "bob".to_string(),
            name: "Bob".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create bob");

    soft_delete_user(project_path, "bob")
        .await
        .expect("soft-delete bob");

    let users = list_users(project_path, None, true)
        .await
        .expect("list with deleted");
    assert_eq!(users.len(), 2);
}

#[tokio::test]
async fn test_list_users_filter_by_git_username() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec!["alice-github".to_string()],
        },
    )
    .await
    .expect("create alice");

    create_user(
        project_path,
        CreateUserOptions {
            id: "bob".to_string(),
            name: "Bob".to_string(),
            email: None,
            git_usernames: vec!["bob-github".to_string()],
        },
    )
    .await
    .expect("create bob");

    let filtered = list_users(project_path, Some("alice-github"), false)
        .await
        .expect("filter");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "alice");
}

#[tokio::test]
async fn test_list_users_filter_no_match() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec!["alice-github".to_string()],
        },
    )
    .await
    .expect("create alice");

    let filtered = list_users(project_path, Some("nobody"), false)
        .await
        .expect("filter");
    assert!(filtered.is_empty());
}

// ---------------------------------------------------------------------------
// delete_user
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_delete_user_found() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create");

    delete_user(project_path, "alice")
        .await
        .expect("Should delete alice");

    let users = read_users(project_path).await.expect("read");
    assert!(users.is_empty(), "User should be permanently deleted");
}

#[tokio::test]
async fn test_delete_user_not_found() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    let result = delete_user(project_path, "ghost").await;
    assert_user_not_found(result.map(|_| ()), "ghost");
}

#[tokio::test]
async fn test_delete_user_not_initialized() {
    let dir = create_test_dir();
    let project_path = dir.path();

    let result = delete_user(project_path, "alice").await;
    assert_not_initialized(&result.map(|_| ()));
}

// ---------------------------------------------------------------------------
// soft_delete_user
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_soft_delete_user_found() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create");

    let result = soft_delete_user(project_path, "alice")
        .await
        .expect("Should soft-delete");

    assert!(
        result.user.deleted_at.is_some(),
        "deleted_at should be set after soft-delete"
    );

    // User still exists in storage
    let users = read_users(project_path).await.expect("read");
    assert_eq!(users.len(), 1);
    assert!(users[0].deleted_at.is_some());
}

#[tokio::test]
async fn test_soft_delete_user_already_deleted() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create");

    soft_delete_user(project_path, "alice")
        .await
        .expect("first soft-delete");

    let second = soft_delete_user(project_path, "alice").await;
    match second {
        Err(UserError::UserAlreadyDeleted(id)) => {
            assert_eq!(id, "alice");
        }
        Err(e) => panic!("Expected UserAlreadyDeleted(alice), got Err: {e}"),
        Ok(_) => panic!("Expected UserAlreadyDeleted(alice), got Ok"),
    }
}

#[tokio::test]
async fn test_soft_delete_user_not_found() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    let result = soft_delete_user(project_path, "nobody").await;
    assert_user_not_found(result.map(|r| r.user), "nobody");
}

#[tokio::test]
async fn test_soft_delete_user_not_initialized() {
    let dir = create_test_dir();
    let project_path = dir.path();

    let result = soft_delete_user(project_path, "alice").await;
    assert_not_initialized(&result.map(|r| r.user));
}

// ---------------------------------------------------------------------------
// restore_user
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_restore_user_found() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create");

    soft_delete_user(project_path, "alice")
        .await
        .expect("soft-delete");

    let result = restore_user(project_path, "alice")
        .await
        .expect("Should restore");

    assert!(
        result.user.deleted_at.is_none(),
        "deleted_at should be cleared after restore"
    );

    let users = read_users(project_path).await.expect("read");
    assert!(users[0].deleted_at.is_none());
}

#[tokio::test]
async fn test_restore_user_not_deleted_error() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create");

    // Try to restore a user that is NOT soft-deleted
    let result = restore_user(project_path, "alice").await;
    match result {
        Err(UserError::UserNotDeleted(id)) => {
            assert_eq!(id, "alice");
        }
        Err(e) => panic!("Expected UserNotDeleted(alice), got Err: {e}"),
        Ok(_) => panic!("Expected UserNotDeleted(alice), got Ok"),
    }
}

#[tokio::test]
async fn test_restore_user_not_found() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    let result = restore_user(project_path, "ghost").await;
    assert_user_not_found(result.map(|r| r.user), "ghost");
}

#[tokio::test]
async fn test_restore_user_not_initialized() {
    let dir = create_test_dir();
    let project_path = dir.path();

    let result = restore_user(project_path, "alice").await;
    assert_not_initialized(&result.map(|r| r.user));
}

// ---------------------------------------------------------------------------
// update_user
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_update_user_name() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create");

    let result = update_user(
        project_path,
        "alice",
        UpdateUserOptions {
            name: Some("Alice Updated".to_string()),
            email: None,
            git_usernames: None,
        },
    )
    .await
    .expect("Should update");

    assert_eq!(result.user.name, "Alice Updated");
}

#[tokio::test]
async fn test_update_user_email_set() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create");

    let result = update_user(
        project_path,
        "alice",
        UpdateUserOptions {
            name: None,
            email: Some("alice@new.example.com".to_string()),
            git_usernames: None,
        },
    )
    .await
    .expect("Should update email");

    assert_eq!(result.user.email.as_deref(), Some("alice@new.example.com"));
}

#[tokio::test]
async fn test_update_user_email_clear_with_empty_string() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: Some("alice@example.com".to_string()),
            git_usernames: vec![],
        },
    )
    .await
    .expect("create");

    // Setting email to empty string should clear it
    let result = update_user(
        project_path,
        "alice",
        UpdateUserOptions {
            name: None,
            email: Some(String::new()),
            git_usernames: None,
        },
    )
    .await
    .expect("Should update");

    assert!(result.user.email.is_none(), "Email should be cleared");
}

#[tokio::test]
async fn test_update_user_git_usernames() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec!["old-handle".to_string()],
        },
    )
    .await
    .expect("create");

    let result = update_user(
        project_path,
        "alice",
        UpdateUserOptions {
            name: None,
            email: None,
            git_usernames: Some(vec!["new-handle".to_string(), "also-alice".to_string()]),
        },
    )
    .await
    .expect("Should update git_usernames");

    assert_eq!(result.user.git_usernames, vec!["new-handle", "also-alice"]);
}

#[tokio::test]
async fn test_update_user_empty_name_no_op() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec![],
        },
    )
    .await
    .expect("create");

    // Empty name string should be a no-op (existing name kept)
    let result = update_user(
        project_path,
        "alice",
        UpdateUserOptions {
            name: Some(String::new()),
            email: None,
            git_usernames: None,
        },
    )
    .await
    .expect("Should update without changing name");

    assert_eq!(result.user.name, "Alice", "Name should remain unchanged");
}

#[tokio::test]
async fn test_update_user_empty_git_usernames_no_op() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: None,
            git_usernames: vec!["alice-git".to_string()],
        },
    )
    .await
    .expect("create");

    // Empty git_usernames vec should be a no-op
    let result = update_user(
        project_path,
        "alice",
        UpdateUserOptions {
            name: None,
            email: None,
            git_usernames: Some(vec![]),
        },
    )
    .await
    .expect("Should update");

    assert_eq!(
        result.user.git_usernames,
        vec!["alice-git"],
        "git_usernames should remain unchanged when empty vec provided"
    );
}

#[tokio::test]
async fn test_update_user_not_found() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    let result = update_user(
        project_path,
        "nobody",
        UpdateUserOptions {
            name: Some("New Name".to_string()),
            email: None,
            git_usernames: None,
        },
    )
    .await;

    assert_user_not_found(result.map(|r| r.user), "nobody");
}

#[tokio::test]
async fn test_update_user_not_initialized() {
    let dir = create_test_dir();
    let project_path = dir.path();

    let result = update_user(
        project_path,
        "alice",
        UpdateUserOptions {
            name: Some("Alice".to_string()),
            email: None,
            git_usernames: None,
        },
    )
    .await;

    assert_not_initialized(&result.map(|r| r.user));
}

#[tokio::test]
async fn test_update_user_not_initialized_manifest_missing_but_centy_dir_exists() {
    // This exercises the `.ok_or(UserError::NotInitialized)` branch in update_user:
    // read_manifest returns Ok(None) (file absent but no IO error), so ok_or fires.
    let dir = create_test_dir();
    let project_path = dir.path();
    // Create the .centy directory but do NOT write a manifest file.
    let centy_dir = project_path.join(".centy");
    std::fs::create_dir_all(&centy_dir).expect("create .centy dir");

    let result = update_user(
        project_path,
        "alice",
        UpdateUserOptions {
            name: Some("Alice".to_string()),
            email: None,
            git_usernames: None,
        },
    )
    .await;

    assert_not_initialized(&result.map(|r| r.user));
}

#[tokio::test]
async fn test_update_user_all_fields() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: Some("old@example.com".to_string()),
            git_usernames: vec!["old-git".to_string()],
        },
    )
    .await
    .expect("create");

    let result = update_user(
        project_path,
        "alice",
        UpdateUserOptions {
            name: Some("Alice Updated".to_string()),
            email: Some("new@example.com".to_string()),
            git_usernames: Some(vec!["new-git".to_string()]),
        },
    )
    .await
    .expect("Should update all fields");

    assert_eq!(result.user.name, "Alice Updated");
    assert_eq!(result.user.email.as_deref(), Some("new@example.com"));
    assert_eq!(result.user.git_usernames, vec!["new-git"]);
}

#[tokio::test]
async fn test_update_user_no_options_no_op() {
    let dir = create_test_dir();
    let project_path = dir.path();
    init_centy_project(project_path).await;

    create_user(
        project_path,
        CreateUserOptions {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            email: Some("alice@example.com".to_string()),
            git_usernames: vec!["alice-git".to_string()],
        },
    )
    .await
    .expect("create");

    let result = update_user(
        project_path,
        "alice",
        UpdateUserOptions {
            name: None,
            email: None,
            git_usernames: None,
        },
    )
    .await
    .expect("No-op update should succeed");

    assert_eq!(result.user.name, "Alice");
    assert_eq!(result.user.email.as_deref(), Some("alice@example.com"));
    assert_eq!(result.user.git_usernames, vec!["alice-git"]);
}
