#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod common;

use centy_daemon::user::{create_user, get_user, sync_users, CreateUserOptions};
use common::{create_test_dir, init_centy_project};
use std::path::Path;
use std::process::Command;

/// Make `dir` a git repo with one commit by Alice.
fn git_repo_with_alice(dir: &Path) {
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env("GIT_AUTHOR_NAME", "Alice")
            .env("GIT_AUTHOR_EMAIL", "alice@example.com")
            .env("GIT_COMMITTER_NAME", "Alice")
            .env("GIT_COMMITTER_EMAIL", "alice@example.com")
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    };
    git(&["init", "-q"]);
    git(&["commit", "-q", "--allow-empty", "-m", "init"]);
}

async fn user_with(dir: &Path, git_usernames: Vec<String>) {
    create_user(
        dir,
        CreateUserOptions {
            id: "alice-user".to_string(),
            name: "Alice Existing".to_string(),
            email: Some("alice@example.com".to_string()),
            git_usernames,
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_sync_fills_missing_git_usernames() {
    let dir = create_test_dir();
    init_centy_project(dir.path()).await;
    git_repo_with_alice(dir.path());
    user_with(dir.path(), vec![]).await;

    let full = sync_users(dir.path(), false).await.unwrap();

    assert_eq!(full.result.updated, vec!["alice-user".to_string()]);
    assert_eq!(full.result.created.len(), 0);
    assert_eq!(full.result.skipped.len(), 0);
    let user = get_user(dir.path(), "alice-user").await.unwrap();
    assert_eq!(user.git_usernames, vec!["Alice".to_string()]);
    assert_eq!(user.name, "Alice Existing");
}

#[tokio::test]
async fn test_sync_dry_run_reports_update_without_writing() {
    let dir = create_test_dir();
    init_centy_project(dir.path()).await;
    git_repo_with_alice(dir.path());
    user_with(dir.path(), vec![]).await;

    let full = sync_users(dir.path(), true).await.unwrap();

    assert_eq!(full.result.would_update.len(), 1);
    assert_eq!(full.result.would_skip.len(), 0);
    assert_eq!(full.result.updated.len(), 0);
    let user = get_user(dir.path(), "alice-user").await.unwrap();
    assert_eq!(user.git_usernames.len(), 0);
}

#[tokio::test]
async fn test_sync_keeps_existing_git_usernames() {
    let dir = create_test_dir();
    init_centy_project(dir.path()).await;
    git_repo_with_alice(dir.path());
    user_with(dir.path(), vec!["alice-handle".to_string()]).await;

    let full = sync_users(dir.path(), false).await.unwrap();

    assert_eq!(full.result.updated.len(), 0);
    assert_eq!(full.result.skipped, vec!["alice@example.com".to_string()]);
    let user = get_user(dir.path(), "alice-user").await.unwrap();
    assert_eq!(user.git_usernames, vec!["alice-handle".to_string()]);
}
