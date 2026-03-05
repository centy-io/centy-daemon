#![allow(clippy::indexing_slicing)]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::await_holding_lock)] // Intentional: serialize tests via std::sync::Mutex

//! Integration tests for organization inference from git remotes.

use centy_daemon::registry::{
    delete_organization, get_organization, infer_organization_from_remote,
};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

// Serialize tests to avoid concurrent registry writes on ~/.centy/projects.json.
static REGISTRY_LOCK: Mutex<()> = Mutex::new(());

// Per-binary isolated temp directory so different test binaries don't race
// on the real ~/.centy/projects.json registry file.
static TEST_HOME: OnceLock<TempDir> = OnceLock::new();

/// Acquire the registry lock and ensure an isolated per-binary registry is set up.
/// Recovers from mutex poison caused by test panics to prevent cascade failures.
fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_HOME.get_or_init(|| {
        let dir = TempDir::new().expect("Failed to create test home dir");
        std::env::set_var("CENTY_HOME", dir.path());
        dir
    });
    REGISTRY_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn create_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Initialize a git repository with a remote
fn setup_git_repo_with_remote(temp_dir: &TempDir, remote_url: &str) {
    let path = temp_dir.path();

    // Initialize git repo
    // Clear GIT_DIR and GIT_WORK_TREE to avoid being affected by git hooks environment
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .expect("Failed to init git repo");

    // Configure git user (required for some git operations)
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .expect("Failed to config git email");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .expect("Failed to config git name");

    // Add remote origin
    Command::new("git")
        .args(["remote", "add", "origin", remote_url])
        .current_dir(path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .expect("Failed to add remote");
}

#[tokio::test]
async fn test_infer_org_from_github_https() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    setup_git_repo_with_remote(&temp_dir, "https://github.com/acme-corp/my-repo.git");

    let result = infer_organization_from_remote(temp_dir.path(), None).await;

    assert_eq!(result.inferred_org_slug, Some("acme-corp".to_string()));
    assert_eq!(result.inferred_org_name, Some("acme-corp".to_string()));
    assert!(!result.has_mismatch);
    assert!(result.existing_org_slug.is_none());

    // Cleanup
    let _ = delete_organization("acme-corp", false).await;
}

#[tokio::test]
async fn test_infer_org_from_github_ssh() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    setup_git_repo_with_remote(&temp_dir, "git@github.com:my-startup/api-service.git");

    let result = infer_organization_from_remote(temp_dir.path(), None).await;

    assert_eq!(result.inferred_org_slug, Some("my-startup".to_string()));
    assert_eq!(result.inferred_org_name, Some("my-startup".to_string()));
    assert!(!result.has_mismatch);

    // Cleanup
    let _ = delete_organization("my-startup", false).await;
}

#[tokio::test]
async fn test_infer_org_from_gitlab_url() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    setup_git_repo_with_remote(&temp_dir, "https://gitlab.com/engineering-team/backend.git");

    let result = infer_organization_from_remote(temp_dir.path(), None).await;

    assert_eq!(
        result.inferred_org_slug,
        Some("engineering-team".to_string())
    );
    assert_eq!(
        result.inferred_org_name,
        Some("engineering-team".to_string())
    );
    assert!(!result.has_mismatch);

    // Cleanup
    let _ = delete_organization("engineering-team", false).await;
}

#[tokio::test]
async fn test_infer_org_from_self_hosted() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    setup_git_repo_with_remote(
        &temp_dir,
        "https://git.company.internal/platform/service.git",
    );

    let result = infer_organization_from_remote(temp_dir.path(), None).await;

    assert_eq!(result.inferred_org_slug, Some("platform".to_string()));
    assert_eq!(result.inferred_org_name, Some("platform".to_string()));
    assert!(!result.has_mismatch);

    // Cleanup
    let _ = delete_organization("platform", false).await;
}

#[tokio::test]
async fn test_infer_org_mismatch_detection() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    setup_git_repo_with_remote(&temp_dir, "git@github.com:new-org/repo.git");

    // Test with an existing org assigned
    let result = infer_organization_from_remote(temp_dir.path(), Some("old-org")).await;

    assert_eq!(result.inferred_org_slug, Some("new-org".to_string()));
    assert_eq!(result.existing_org_slug, Some("old-org".to_string()));
    assert!(result.has_mismatch);
    assert!(result.message.unwrap().contains("but git remote suggests"));

    // Cleanup - only delete if it was created
    let _ = delete_organization("new-org", false).await;
}

#[tokio::test]
async fn test_infer_org_no_mismatch_when_same() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    setup_git_repo_with_remote(&temp_dir, "git@github.com:same-org/repo.git");

    // Test with same org already assigned
    let result = infer_organization_from_remote(temp_dir.path(), Some("same-org")).await;

    assert_eq!(result.inferred_org_slug, Some("same-org".to_string()));
    assert_eq!(result.existing_org_slug, Some("same-org".to_string()));
    assert!(!result.has_mismatch);

    // Cleanup
    let _ = delete_organization("same-org", false).await;
}

#[tokio::test]
async fn test_infer_org_non_git_directory() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    // Don't init git - just a plain directory

    let result = infer_organization_from_remote(temp_dir.path(), None).await;

    assert!(result.inferred_org_slug.is_none());
    assert!(result.inferred_org_name.is_none());
    assert!(!result.has_mismatch);
    assert!(result.message.unwrap().contains("Not a git repository"));
}

#[tokio::test]
async fn test_infer_org_git_no_remote() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();

    // Initialize git repo but don't add a remote
    // Clear GIT_DIR to avoid being affected by git hooks environment
    Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .expect("Failed to init git repo");

    let result = infer_organization_from_remote(temp_dir.path(), None).await;

    assert!(result.inferred_org_slug.is_none());
    assert!(result.inferred_org_name.is_none());
    assert!(!result.has_mismatch);
    assert!(result.message.unwrap().contains("No origin remote found"));
}

#[tokio::test]
async fn test_infer_org_creates_organization() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    setup_git_repo_with_remote(&temp_dir, "https://github.com/auto-create-test/repo.git");

    // First ensure the org doesn't exist
    let _ = delete_organization("auto-create-test", false).await;

    let result = infer_organization_from_remote(temp_dir.path(), None).await;

    assert_eq!(
        result.inferred_org_slug,
        Some("auto-create-test".to_string())
    );
    assert!(result.org_created);
    assert!(result.message.unwrap().contains("Created organization"));

    // Verify org was actually created
    let org = get_organization("auto-create-test")
        .await
        .expect("Should get org");
    assert!(org.is_some());
    assert_eq!(org.unwrap().slug, "auto-create-test");

    // Cleanup
    let _ = delete_organization("auto-create-test", false).await;
}

#[tokio::test]
async fn test_infer_org_uses_existing_organization() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    setup_git_repo_with_remote(&temp_dir, "https://github.com/existing-org-test/repo.git");

    // Cleanup any leftover from previous test runs
    let _ = delete_organization("existing-org-test", false).await;

    // Pre-create the organization
    centy_daemon::registry::create_organization(
        Some("existing-org-test"),
        "Existing Org Test",
        Some("Pre-existing org"),
    )
    .await
    .expect("Should create org");

    let result = infer_organization_from_remote(temp_dir.path(), None).await;

    assert_eq!(
        result.inferred_org_slug,
        Some("existing-org-test".to_string())
    );
    assert!(!result.org_created); // Should NOT create a new one
    assert!(result
        .message
        .unwrap()
        .contains("Using existing organization"));

    // Cleanup
    let _ = delete_organization("existing-org-test", false).await;
}

#[tokio::test]
async fn test_infer_org_slugifies_name() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    // Use an org name that needs slugification
    setup_git_repo_with_remote(&temp_dir, "https://github.com/My-ORG_Name/repo.git");

    let result = infer_organization_from_remote(temp_dir.path(), None).await;

    // The org name in URL is "My-ORG_Name" but slugified should be lowercase with hyphens
    assert_eq!(result.inferred_org_slug, Some("my-org-name".to_string()));

    // Cleanup
    let _ = delete_organization("my-org-name", false).await;
}
