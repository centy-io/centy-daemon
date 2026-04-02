#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::await_holding_lock)]

use super::*;
use crate::registry::storage::{get_lock, read_registry, write_registry_unlocked};
use crate::registry::types::{Organization, TrackedProject};
use tempfile::TempDir;

// Use the shared process-wide lock via the tracking module.
fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
    crate::registry::tracking::acquire_tracking_test_lock()
}

/// Insert a tracked project directly into the registry for test setup.
/// Uses the async lock from `set_ops` itself to avoid deadlocks.
async fn insert_tracked_project_locked(path: &str, project: TrackedProject) {
    let _guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read_registry failed");
    registry.projects.insert(path.to_string(), project);
    write_registry_unlocked(&registry)
        .await
        .expect("write_registry_unlocked failed");
}

fn make_tracked_project() -> TrackedProject {
    TrackedProject {
        first_accessed: "2024-01-01T00:00:00Z".to_string(),
        last_accessed: "2024-01-01T00:00:00Z".to_string(),
        is_favorite: false,
        is_archived: false,
        organization_slug: None,
        user_title: None,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// set_project_favorite tests
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_set_project_favorite_not_found() {
    let _lock = acquire_lock();
    let result = set_project_favorite("/nonexistent/path/abc123xyz", true).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_project_favorite_true() {
    let _lock = acquire_lock();
    let dir = TempDir::new().expect("tempdir");
    // Use the canonical path (macOS: /private/var/... vs /var/...)
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    insert_tracked_project_locked(&path, make_tracked_project()).await;

    let info = set_project_favorite(&path, true)
        .await
        .expect("Should succeed");
    assert!(info.is_favorite);
}

#[tokio::test]
async fn test_set_project_favorite_false() {
    let _lock = acquire_lock();
    let dir = TempDir::new().expect("tempdir");
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    let mut tracked = make_tracked_project();
    tracked.is_favorite = true;
    insert_tracked_project_locked(&path, tracked).await;

    let info = set_project_favorite(&path, false)
        .await
        .expect("Should succeed");
    assert!(!info.is_favorite);
}

#[tokio::test]
async fn test_set_project_favorite_with_org() {
    let _lock = acquire_lock();
    let dir = TempDir::new().expect("tempdir");
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    // Set up registry with an org and a project that references it
    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    let org_slug = "test-fav-org".to_string();
    registry.organizations.insert(
        org_slug.clone(),
        Organization {
            name: "Test Fav Org".to_string(),
            description: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        },
    );
    let mut tracked = make_tracked_project();
    tracked.organization_slug = Some(org_slug.clone());
    registry.projects.insert(path.clone(), tracked);
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let info = set_project_favorite(&path, true)
        .await
        .expect("Should succeed");
    assert!(info.is_favorite);
    assert_eq!(info.organization_slug, Some(org_slug.clone()));
    assert_eq!(info.organization_name, Some("Test Fav Org".to_string()));
}

#[tokio::test]
async fn test_set_project_favorite_org_not_in_registry() {
    let _lock = acquire_lock();
    let dir = TempDir::new().expect("tempdir");
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    // Project references a slug that doesn't exist in organizations map
    let mut tracked = make_tracked_project();
    tracked.organization_slug = Some("missing-org".to_string());
    insert_tracked_project_locked(&path, tracked).await;

    let info = set_project_favorite(&path, true)
        .await
        .expect("Should succeed");
    // org_name should be None since the slug doesn't resolve
    assert_eq!(info.organization_name, None);
}

// ──────────────────────────────────────────────────────────────────────────────
// set_project_archived tests
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_set_project_archived_not_found() {
    let _lock = acquire_lock();
    let result = set_project_archived("/nonexistent/path/xyz999abc", true).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_project_archived_true() {
    let _lock = acquire_lock();
    let dir = TempDir::new().expect("tempdir");
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    insert_tracked_project_locked(&path, make_tracked_project()).await;

    let info = set_project_archived(&path, true)
        .await
        .expect("Should succeed");
    assert!(info.is_archived);
}

#[tokio::test]
async fn test_set_project_archived_false() {
    let _lock = acquire_lock();
    let dir = TempDir::new().expect("tempdir");
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    let mut tracked = make_tracked_project();
    tracked.is_archived = true;
    insert_tracked_project_locked(&path, tracked).await;

    let info = set_project_archived(&path, false)
        .await
        .expect("Should succeed");
    assert!(!info.is_archived);
}

#[tokio::test]
async fn test_set_project_archived_with_org() {
    let _lock = acquire_lock();
    let dir = TempDir::new().expect("tempdir");
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    let org_slug = "test-arch-org".to_string();
    registry.organizations.insert(
        org_slug.clone(),
        Organization {
            name: "Test Arch Org".to_string(),
            description: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        },
    );
    let mut tracked = make_tracked_project();
    tracked.organization_slug = Some(org_slug.clone());
    registry.projects.insert(path.clone(), tracked);
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let info = set_project_archived(&path, true)
        .await
        .expect("Should succeed");
    assert!(info.is_archived);
    assert_eq!(info.organization_name, Some("Test Arch Org".to_string()));
}

// ──────────────────────────────────────────────────────────────────────────────
// set_project_user_title tests
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_set_project_user_title_not_found() {
    let _lock = acquire_lock();
    let result =
        set_project_user_title("/nonexistent/path/uvw888abc", Some("My Title".to_string())).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_project_user_title_some() {
    let _lock = acquire_lock();
    let dir = TempDir::new().expect("tempdir");
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    insert_tracked_project_locked(&path, make_tracked_project()).await;

    let info = set_project_user_title(&path, Some("Custom Title".to_string()))
        .await
        .expect("Should succeed");
    assert_eq!(info.user_title, Some("Custom Title".to_string()));
}

#[tokio::test]
async fn test_set_project_user_title_none() {
    let _lock = acquire_lock();
    let dir = TempDir::new().expect("tempdir");
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    let mut tracked = make_tracked_project();
    tracked.user_title = Some("Old Title".to_string());
    insert_tracked_project_locked(&path, tracked).await;

    let info = set_project_user_title(&path, None)
        .await
        .expect("Should succeed");
    assert_eq!(info.user_title, None);
}

#[tokio::test]
async fn test_set_project_user_title_empty_string_becomes_none() {
    let _lock = acquire_lock();
    let dir = TempDir::new().expect("tempdir");
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    let mut tracked = make_tracked_project();
    tracked.user_title = Some("Had Title".to_string());
    insert_tracked_project_locked(&path, tracked).await;

    let info = set_project_user_title(&path, Some(String::new()))
        .await
        .expect("Should succeed");
    // Empty string should become None (filtered out)
    assert_eq!(info.user_title, None);
}

#[tokio::test]
async fn test_set_project_user_title_with_org() {
    let _lock = acquire_lock();
    let dir = TempDir::new().expect("tempdir");
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    let org_slug = "test-title-org".to_string();
    registry.organizations.insert(
        org_slug.clone(),
        Organization {
            name: "Test Title Org".to_string(),
            description: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        },
    );
    let mut tracked = make_tracked_project();
    tracked.organization_slug = Some(org_slug.clone());
    registry.projects.insert(path.clone(), tracked);
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let info = set_project_user_title(&path, Some("My Title".to_string()))
        .await
        .expect("Should succeed");
    assert_eq!(info.user_title, Some("My Title".to_string()));
    assert_eq!(info.organization_name, Some("Test Title Org".to_string()));
}

// ──────────────────────────────────────────────────────────────────────────────
// Fallback path (non-canonical path lookup) tests
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_set_project_favorite_fallback_path() {
    let _lock = acquire_lock();
    // Use a path that won't canonicalize (doesn't exist) — stored as-is, looked up via fallback
    let path = "/tmp/nonexistent-centy-set-ops-test-abc/proj".to_string();

    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    registry
        .projects
        .insert(path.clone(), make_tracked_project());
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let info = set_project_favorite(&path, true)
        .await
        .expect("Should succeed via fallback path lookup");
    assert!(info.is_favorite);
}

#[tokio::test]
async fn test_set_project_archived_fallback_path() {
    let _lock = acquire_lock();
    let path = "/tmp/nonexistent-centy-set-ops-test-def/proj".to_string();

    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    registry
        .projects
        .insert(path.clone(), make_tracked_project());
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let info = set_project_archived(&path, true)
        .await
        .expect("Should succeed via fallback path lookup");
    assert!(info.is_archived);
}

#[tokio::test]
async fn test_set_project_user_title_fallback_path() {
    let _lock = acquire_lock();
    let path = "/tmp/nonexistent-centy-set-ops-test-ghi/proj".to_string();

    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    registry
        .projects
        .insert(path.clone(), make_tracked_project());
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let info = set_project_user_title(&path, Some("Fallback Title".to_string()))
        .await
        .expect("Should succeed via fallback path lookup");
    assert_eq!(info.user_title, Some("Fallback Title".to_string()));
}

// ──────────────────────────────────────────────────────────────────────────────
// Symlink tests: exercises the `else if` fallback branch
// canonical_path resolves the symlink → differs from project_path (symlink) →
// first get_mut(canonical) misses, else-if get_mut(project_path) hits.
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(unix)]
#[tokio::test]
async fn test_set_project_favorite_else_if_symlink_branch() {
    use std::os::unix::fs::symlink;
    let _lock = acquire_lock();

    let real_dir = TempDir::new().expect("real tempdir");
    let link_parent = TempDir::new().expect("link parent tempdir");
    let symlink_path = link_parent.path().join("link_proj");
    symlink(real_dir.path(), &symlink_path).expect("create symlink");
    let symlink_str = symlink_path.to_string_lossy().to_string();

    // Verify the symlink resolves to a different canonical path.
    let resolved = std::path::Path::new(&symlink_str)
        .canonicalize()
        .expect("canonicalize symlink");
    if resolved.to_string_lossy() == symlink_str {
        // No symlink effect on this platform/config – skip.
        return;
    }

    // Store under the symlink path (not the canonical path).
    insert_tracked_project_locked(&symlink_str, make_tracked_project()).await;

    // canonical_path = resolved ≠ symlink_str → first get_mut misses, else-if hits.
    let info = set_project_favorite(&symlink_str, true)
        .await
        .expect("Should succeed via else-if symlink fallback");
    assert!(info.is_favorite);
}

#[cfg(unix)]
#[tokio::test]
async fn test_set_project_archived_else_if_symlink_branch() {
    use std::os::unix::fs::symlink;
    let _lock = acquire_lock();

    let real_dir = TempDir::new().expect("real tempdir");
    let link_parent = TempDir::new().expect("link parent tempdir");
    let symlink_path = link_parent.path().join("link_arch");
    symlink(real_dir.path(), &symlink_path).expect("create symlink");
    let symlink_str = symlink_path.to_string_lossy().to_string();

    let resolved = std::path::Path::new(&symlink_str)
        .canonicalize()
        .expect("canonicalize symlink");
    if resolved.to_string_lossy() == symlink_str {
        return;
    }

    insert_tracked_project_locked(&symlink_str, make_tracked_project()).await;

    let info = set_project_archived(&symlink_str, true)
        .await
        .expect("Should succeed via else-if symlink fallback");
    assert!(info.is_archived);
}

#[cfg(unix)]
#[tokio::test]
async fn test_set_project_user_title_else_if_symlink_branch() {
    use std::os::unix::fs::symlink;
    let _lock = acquire_lock();

    let real_dir = TempDir::new().expect("real tempdir");
    let link_parent = TempDir::new().expect("link parent tempdir");
    let symlink_path = link_parent.path().join("link_title");
    symlink(real_dir.path(), &symlink_path).expect("create symlink");
    let symlink_str = symlink_path.to_string_lossy().to_string();

    let resolved = std::path::Path::new(&symlink_str)
        .canonicalize()
        .expect("canonicalize symlink");
    if resolved.to_string_lossy() == symlink_str {
        return;
    }

    insert_tracked_project_locked(&symlink_str, make_tracked_project()).await;

    let info = set_project_user_title(&symlink_str, Some("Symlink Title".to_string()))
        .await
        .expect("Should succeed via else-if symlink fallback");
    assert_eq!(info.user_title, Some("Symlink Title".to_string()));
}

// ──────────────────────────────────────────────────────────────────────────────
// Invalid-manifest test: exercises the `_ => (None, false)` arm in enrich_fn.rs
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_enrich_project_invalid_manifest_gives_no_version() {
    let _lock = acquire_lock();

    let dir = TempDir::new().expect("tempdir");
    // Create .centy/ dir with an INVALID manifest so initialized=true but read fails.
    let centy_path = dir.path().join(".centy");
    std::fs::create_dir_all(&centy_path).expect("create .centy");
    std::fs::write(centy_path.join(".centy-manifest.json"), b"not valid json { }")
        .expect("write invalid manifest");

    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    insert_tracked_project_locked(&path, make_tracked_project()).await;

    // set_project_favorite calls enrich_project → hits `_ => (None, false)`
    let info = set_project_favorite(&path, false)
        .await
        .expect("Should succeed despite invalid manifest");

    // project_version should be None (from the `_` arm of the manifest match)
    assert!(info.project_version.is_none());
    assert!(!info.project_behind);
}
