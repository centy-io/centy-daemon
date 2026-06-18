//! Tests for `assignment::set_project_organization` covering missed branches.
//!
//! `super` here is `assignment`, `super::super` is `organizations`.
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::await_holding_lock)]

use super::set_project_organization;
use crate::registry::storage::{get_lock, read_registry, write_registry_unlocked};
use crate::registry::types::{Organization, TrackedProject};
use crate::utils::{get_centy_path, now_iso};
use tempfile::TempDir;

fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
    super::super::acquire_org_test_lock()
}

async fn insert_org(slug: &str, name: &str) {
    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    registry.organizations.insert(
        slug.to_string(),
        Organization {
            name: name.to_string(),
            description: None,
            created_at: now_iso(),
            updated_at: now_iso(),
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);
}

async fn insert_project(canonical: &str) {
    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    registry.projects.insert(
        canonical.to_string(),
        TrackedProject {
            first_accessed: now_iso(),
            last_accessed: now_iso(),
            is_favorite: false,
            is_archived: false,
            organization_slug: None,
            user_title: None,
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);
}

/// Covers the `slug.is_empty()` branch in `verify_org_and_name` — empty slug
/// acts like None (removes org assignment).
#[tokio::test]
async fn test_set_project_organization_empty_slug_treated_as_none() {
    let _lock = acquire_lock();

    let project_dir = TempDir::new().expect("project tmp");
    let path_str = project_dir.path().to_string_lossy().to_string();

    insert_project(&path_str).await;

    let result = set_project_organization(&path_str, Some("")).await;
    assert!(result.is_ok(), "empty slug should succeed, got: {result:?}");
    let info = result.unwrap();
    assert!(info.organization_slug.is_none());
}

/// Covers the `else if org_file_path.exists()` branch — passing None when an
/// org file is already present on disk should remove it.
#[tokio::test]
async fn test_set_project_organization_none_removes_existing_org_file() {
    let _lock = acquire_lock();

    let project_dir = TempDir::new().expect("project tmp");
    let path_str = project_dir.path().to_string_lossy().to_string();

    insert_project(&path_str).await;

    // Manually create an org file so it "exists".
    let centy = get_centy_path(project_dir.path());
    tokio::fs::create_dir_all(&centy)
        .await
        .expect("create .centy");
    let org_file = centy.join("organization.json");
    tokio::fs::write(&org_file, b"{\"slug\":\"x\",\"name\":\"X\"}")
        .await
        .expect("write org file");
    assert!(org_file.exists(), "org file should exist before test");

    let result = set_project_organization(&path_str, None).await;
    assert!(result.is_ok(), "should succeed removing org");

    assert!(!org_file.exists(), "org file should have been removed");
}

/// Covers the `else { // no org file to remove }` branch — passing None when
/// there is NO org file on disk should also succeed.
#[tokio::test]
async fn test_set_project_organization_none_no_org_file_succeeds() {
    let _lock = acquire_lock();

    let project_dir = TempDir::new().expect("project tmp");
    let path_str = project_dir.path().to_string_lossy().to_string();

    insert_project(&path_str).await;

    let result = set_project_organization(&path_str, None).await;
    assert!(result.is_ok(), "should succeed when no org file to remove");
    let info = result.unwrap();
    assert!(info.organization_slug.is_none());
}

/// Covers the `set_project_organization` with a valid org slug.
#[tokio::test]
async fn test_set_project_organization_assigns_org() {
    let _lock = acquire_lock();

    let project_dir = TempDir::new().expect("project tmp");
    let path_str = project_dir.path().to_string_lossy().to_string();

    insert_project(&path_str).await;
    insert_org("assign-my-org", "Assign My Org").await;

    let result = set_project_organization(&path_str, Some("assign-my-org")).await;
    assert!(result.is_ok(), "should assign org: {result:?}");
    let info = result.unwrap();
    assert_eq!(info.organization_slug.as_deref(), Some("assign-my-org"));
}

/// Covers the `OrganizationError::NotFound` branch in `verify_org_and_name`.
#[tokio::test]
async fn test_set_project_organization_not_found_org() {
    let _lock = acquire_lock();

    let project_dir = TempDir::new().expect("project tmp");
    let path_str = project_dir.path().to_string_lossy().to_string();

    insert_project(&path_str).await;

    let result = set_project_organization(&path_str, Some("ghost-org-xyz-not-exists")).await;
    assert!(result.is_err(), "should fail for non-existent org");
}

/// Covers the `or_insert_with` closure — project is NOT pre-existing in the registry,
/// so `set_project_organization` auto-creates the `TrackedProject` entry.
#[tokio::test]
async fn test_set_project_organization_auto_creates_project_entry() {
    let _lock = acquire_lock();

    let project_dir = TempDir::new().expect("project tmp");
    let path_str = project_dir.path().to_string_lossy().to_string();

    // Do NOT call insert_project — the project has never been tracked.
    // set_project_organization should create a new TrackedProject via or_insert_with.
    let result = set_project_organization(&path_str, None).await;
    assert!(
        result.is_ok(),
        "should auto-create project entry: {result:?}"
    );
    let info = result.unwrap();
    assert!(info.organization_slug.is_none());
}

/// Covers the `write_project_org_file` branch — assigns an org to an untracked project,
/// exercising the org-file write path (lines 75-82) with the `.centy` dir creation.
#[tokio::test]
async fn test_set_project_organization_writes_org_file() {
    let _lock = acquire_lock();

    let project_dir = TempDir::new().expect("project tmp");
    let path_str = project_dir.path().to_string_lossy().to_string();

    // Create the .centy dir so the org file can be written.
    let centy = get_centy_path(project_dir.path());
    tokio::fs::create_dir_all(&centy)
        .await
        .expect("create .centy");

    insert_org("write-org-file-org", "Write Org File Org").await;

    let result = set_project_organization(&path_str, Some("write-org-file-org")).await;
    assert!(result.is_ok(), "should write org file: {result:?}");

    // Verify the org file was actually created.
    let org_file = centy.join("organization.json");
    assert!(org_file.exists(), "org file should have been written");
}
