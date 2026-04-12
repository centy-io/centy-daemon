#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::reconciliation::{execute_reconciliation, ReconciliationDecisions};
use tempfile::tempdir;

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

// ---------------------------------------------------------------------------
// Async storage tests: read_users / write_users
// ---------------------------------------------------------------------------

async fn init_project(project_path: &std::path::Path) {
    execute_reconciliation(project_path, ReconciliationDecisions::default(), true)
        .await
        .expect("Failed to initialize project");
}

#[tokio::test]
async fn test_read_users_not_initialized_returns_error() {
    let dir = tempdir().expect("tempdir");
    let project_path = dir.path();

    // No .centy directory → NotInitialized
    let result = read_users(project_path).await;
    assert!(
        matches!(result, Err(UserError::NotInitialized)),
        "Expected NotInitialized error"
    );
}

#[tokio::test]
async fn test_read_users_initialized_no_file_returns_empty() {
    let dir = tempdir().expect("tempdir");
    let project_path = dir.path();
    init_project(project_path).await;

    // No users.json file yet → empty vec
    let users = read_users(project_path).await.expect("Should read");
    assert!(users.is_empty());
}

#[tokio::test]
async fn test_write_and_read_users_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let project_path = dir.path();
    init_project(project_path).await;

    let written = vec![
        make_user("alice", "Alice", Some("alice@example.com")),
        make_user("bob", "Bob", None),
    ];

    write_users(project_path, &written)
        .await
        .expect("Should write users");

    let loaded = read_users(project_path).await.expect("Should read users");
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].id, "alice");
    assert_eq!(loaded[0].email.as_deref(), Some("alice@example.com"));
    assert_eq!(loaded[1].id, "bob");
    assert!(loaded[1].email.is_none());
}

#[tokio::test]
async fn test_write_empty_users_list() {
    let dir = tempdir().expect("tempdir");
    let project_path = dir.path();
    init_project(project_path).await;

    write_users(project_path, &[])
        .await
        .expect("Should write empty list");

    let loaded = read_users(project_path).await.expect("Should read");
    assert!(loaded.is_empty());
}

#[tokio::test]
async fn test_read_users_invalid_manifest_returns_not_initialized() {
    let dir = tempdir().expect("tempdir");
    let project_path = dir.path();

    // Create .centy/ dir with invalid manifest JSON → read_manifest returns Err
    // → map_err closure executes → NotInitialized returned
    let centy_path = project_path.join(".centy");
    std::fs::create_dir_all(&centy_path).expect("create .centy");
    std::fs::write(
        centy_path.join(".centy-manifest.json"),
        b"not valid json { }",
    )
    .expect("write invalid manifest");

    let result = read_users(project_path).await;
    assert!(
        matches!(result, Err(UserError::NotInitialized)),
        "Expected NotInitialized when manifest is invalid"
    );
}

#[tokio::test]
async fn test_read_users_malformed_json_returns_error() {
    let dir = tempdir().expect("tempdir");
    let project_path = dir.path();
    init_project(project_path).await;

    // Write malformed JSON to users.json → serde_json::from_str fails
    let users_path = project_path.join(".centy").join("users.json");
    std::fs::write(&users_path, b"{ not: valid json }").expect("write malformed json");

    let result = read_users(project_path).await;
    assert!(
        result.is_err(),
        "Expected error on malformed users.json, got: {result:?}"
    );
}

#[tokio::test]
async fn test_write_users_overwrites_previous() {
    let dir = tempdir().expect("tempdir");
    let project_path = dir.path();
    init_project(project_path).await;

    let first = vec![make_user("alice", "Alice", None)];
    write_users(project_path, &first)
        .await
        .expect("First write");

    let second = vec![make_user("bob", "Bob", None)];
    write_users(project_path, &second)
        .await
        .expect("Second write");

    let loaded = read_users(project_path).await.expect("Read");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, "bob");
}
