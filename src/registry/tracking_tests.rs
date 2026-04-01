#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::await_holding_lock)]

use super::super::storage::{get_lock, read_registry, write_registry_unlocked};
use super::super::types::{ListProjectsOptions, Organization, TrackedProject};
use super::*;
use tempfile::TempDir;

// Use the shared process-wide lock from the tracking module so that
// set_ops_tests and tracking_tests don't race each other.
fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
    acquire_tracking_test_lock()
}

/// Create a project directory that is NOT in the system temp dir, so
/// `track_project` doesn't skip it via `is_ignored_path`.
/// Uses `current_dir` (the crate source tree) with a unique subdir.
fn make_non_temp_project_dir(name: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    // Use /var/db on macOS as a non-temp area, or fall back to /usr/local
    // Actually, just use current_dir which is the worktree
    let cwd = std::env::current_dir().expect("current_dir");
    let project_dir = cwd.join(format!(".centy-test-{name}-{pid}"));
    std::fs::create_dir_all(&project_dir).expect("create project dir");
    project_dir.canonicalize().expect("canonicalize")
}

fn remove_non_temp_project_dir(path: &str) {
    drop(std::fs::remove_dir_all(path));
}

// ──────────────────────────────────────────────────────────────────────────────
// enrich_fn::is_version_behind tests
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn test_is_version_behind_older() {
    assert!(is_version_behind("0.7.0", "0.8.0"));
}

#[test]
fn test_is_version_behind_same() {
    assert!(!is_version_behind("0.8.0", "0.8.0"));
}

#[test]
fn test_is_version_behind_newer() {
    assert!(!is_version_behind("0.9.0", "0.8.0"));
}

#[test]
fn test_is_version_behind_invalid() {
    assert!(!is_version_behind("invalid", "0.8.0"));
    assert!(!is_version_behind("0.8.0", "invalid"));
}

// ──────────────────────────────────────────────────────────────────────────────
// ops.rs: track_project, track_project_async, untrack_project
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_track_project_new_entry() {
    let _lock = acquire_lock();
    let project_dir = make_non_temp_project_dir("track-new");
    let path = project_dir.to_string_lossy().to_string();

    track_project(&path).await.expect("Should track");

    let info = get_project_info(&path)
        .await
        .expect("read ok")
        .expect("should be tracked");
    assert_eq!(info.path, path);

    drop(untrack_project(&path).await);
    remove_non_temp_project_dir(&path);
}

#[tokio::test]
async fn test_track_project_updates_last_accessed() {
    let _lock = acquire_lock();
    let project_dir = make_non_temp_project_dir("track-update");
    let path = project_dir.to_string_lossy().to_string();

    track_project(&path).await.expect("first track");
    let first = get_project_info(&path)
        .await
        .expect("read ok")
        .expect("tracked");
    let first_time = first.last_accessed.clone();

    // Small delay to ensure time changes
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    track_project(&path).await.expect("second track");
    let second = get_project_info(&path)
        .await
        .expect("read ok")
        .expect("tracked");

    // Both are valid timestamps; second >= first
    assert!(second.last_accessed >= first_time);

    drop(untrack_project(&path).await);
    remove_non_temp_project_dir(&path);
}

#[tokio::test]
async fn test_track_project_async() {
    let _lock = acquire_lock();
    let project_dir = make_non_temp_project_dir("track-async");
    let path = project_dir.to_string_lossy().to_string();

    // track_project_async is fire-and-forget; just verify it doesn't panic
    track_project_async(path.clone());
    // Give the spawned task time to complete
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // cleanup (may or may not be tracked by now)
    drop(untrack_project(&path).await);
    remove_non_temp_project_dir(&path);
}

#[tokio::test]
async fn test_untrack_project_not_found() {
    let _lock = acquire_lock();
    let result = untrack_project("/nonexistent/path/never-tracked-xyz123").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_untrack_project_success() {
    let _lock = acquire_lock();
    let project_dir = make_non_temp_project_dir("untrack-success");
    let path = project_dir.to_string_lossy().to_string();

    track_project(&path).await.expect("should track");
    untrack_project(&path).await.expect("should untrack");

    let info = get_project_info(&path).await.expect("read ok");
    assert!(info.is_none());

    remove_non_temp_project_dir(&path);
}

#[tokio::test]
async fn test_track_project_existing_with_org_no_inference() {
    let _lock = acquire_lock();
    let project_dir = make_non_temp_project_dir("track-with-org");
    let path = project_dir.to_string_lossy().to_string();

    // Pre-insert a project entry that already has an organization_slug set,
    // so that `needs_org_inference` will be false (the `entry.organization_slug.is_none()`
    // branch returns false).
    let g = get_lock().lock().await;
    let mut reg = read_registry().await.expect("read");
    reg.projects.insert(
        path.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: Some("pre-assigned-org".to_string()),
            user_title: None,
        },
    );
    write_registry_unlocked(&reg).await.expect("write");
    drop(g);

    // Tracking should succeed and update last_accessed without triggering inference
    track_project(&path).await.expect("Should track");

    let info = get_project_info(&path)
        .await
        .expect("read ok")
        .expect("tracked");
    // Organization slug should still be set
    assert_eq!(info.organization_slug.as_deref(), Some("pre-assigned-org"));

    drop(untrack_project(&path).await);
    remove_non_temp_project_dir(&path);
}

#[tokio::test]
async fn test_track_project_ignored_path_is_skipped() {
    let _lock = acquire_lock();

    // A path inside temp dir is ignored (falls back to is_in_temp_dir when IGNORE_PREFIXES unset)
    let temp_subpath = std::env::temp_dir()
        .join("centy-test-track-ignored-unique-12345")
        .to_string_lossy()
        .to_string();

    // Ensure the path is NOT already in the registry (from a previous test run)
    let g = get_lock().lock().await;
    let mut reg = read_registry().await.expect("read");
    reg.projects.remove(&temp_subpath);
    write_registry_unlocked(&reg).await.expect("write");
    drop(g);

    // Should succeed without error (early return for ignored paths)
    let result = track_project(&temp_subpath).await;
    assert!(result.is_ok());
    // The project should NOT be in the registry (it was ignored)
    let info = get_project_info(&temp_subpath).await.expect("read ok");
    assert!(info.is_none());
}

// ──────────────────────────────────────────────────────────────────────────────
// enrich_lookups.rs: get_project_info
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_project_info_not_found() {
    let _lock = acquire_lock();
    let info = get_project_info("/path/that/does/not/exist/at/all/xyz789")
        .await
        .expect("Should complete without error");
    assert!(info.is_none());
}

#[tokio::test]
async fn test_get_project_info_found() {
    let _lock = acquire_lock();
    // Use a non-temp dir so track_project doesn't skip it via is_ignored_path
    let project_dir = make_non_temp_project_dir("get-proj-info");
    let path = project_dir.to_string_lossy().to_string();

    track_project(&path).await.expect("should track");

    let info = get_project_info(&path)
        .await
        .expect("Should succeed")
        .expect("Should find project");

    assert_eq!(info.path, path);
    assert!(!info.first_accessed.is_empty());
    assert!(!info.last_accessed.is_empty());

    drop(untrack_project(&path).await);
    remove_non_temp_project_dir(&path);
}

#[tokio::test]
async fn test_get_project_info_with_org() {
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
    let org_slug = "lookup-org".to_string();
    registry.organizations.insert(
        org_slug.clone(),
        Organization {
            name: "Lookup Org".to_string(),
            description: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        },
    );
    registry.projects.insert(
        path.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: Some(org_slug.clone()),
            user_title: None,
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let info = get_project_info(&path)
        .await
        .expect("Should succeed")
        .expect("Should find project");

    assert_eq!(info.organization_slug, Some(org_slug));
    assert_eq!(info.organization_name, Some("Lookup Org".to_string()));
}

// ──────────────────────────────────────────────────────────────────────────────
// enrich.rs: list_projects, get_org_projects
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_projects_empty() {
    let _lock = acquire_lock();

    // Clear all projects first
    let g = get_lock().lock().await;
    let mut reg = read_registry().await.expect("read");
    reg.projects.clear();
    write_registry_unlocked(&reg).await.expect("write");
    drop(g);

    let opts = ListProjectsOptions {
        include_stale: true,
        include_uninitialized: true,
        include_archived: true,
        include_temp: true,
        ..Default::default()
    };
    let projects = list_projects(opts).await.expect("Should succeed");
    assert!(projects.is_empty());
}

#[tokio::test]
async fn test_list_projects_include_stale_false() {
    let _lock = acquire_lock();

    // Insert a project with a path that doesn't exist on disk (stale)
    let stale_path = "/nonexistent/stale/project/abc123".to_string();
    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    registry.projects.insert(
        stale_path.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: None,
            user_title: None,
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let opts = ListProjectsOptions {
        include_stale: false,
        include_uninitialized: true,
        ..Default::default()
    };
    let projects = list_projects(opts).await.expect("Should succeed");
    assert!(!projects.iter().any(|p| p.path == stale_path));
}

#[tokio::test]
async fn test_list_projects_include_stale_true() {
    let _lock = acquire_lock();

    let stale_path = "/nonexistent/stale/project/def456".to_string();
    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    registry.projects.insert(
        stale_path.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: None,
            user_title: None,
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let opts = ListProjectsOptions {
        include_stale: true,
        include_uninitialized: true,
        include_archived: true,
        include_temp: true,
        ..Default::default()
    };
    let projects = list_projects(opts).await.expect("Should succeed");
    assert!(projects.iter().any(|p| p.path == stale_path));
}

#[tokio::test]
async fn test_list_projects_exclude_archived() {
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
    registry.projects.insert(
        path.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: true,
            organization_slug: None,
            user_title: None,
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    // Without include_archived, archived project should be excluded
    let opts = ListProjectsOptions {
        include_stale: true,
        include_uninitialized: true,
        include_archived: false,
        ..Default::default()
    };
    let projects = list_projects(opts).await.expect("Should succeed");
    assert!(!projects.iter().any(|p| p.path == path));
}

#[tokio::test]
async fn test_list_projects_include_archived() {
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
    registry.projects.insert(
        path.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: true,
            organization_slug: None,
            user_title: None,
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let opts = ListProjectsOptions {
        include_stale: true,
        include_uninitialized: true,
        include_archived: true,
        include_temp: true, // project dir is in temp
        ..Default::default()
    };
    let projects = list_projects(opts).await.expect("Should succeed");
    assert!(projects.iter().any(|p| p.path == path));
}

#[tokio::test]
async fn test_list_projects_filter_by_org_slug() {
    let _lock = acquire_lock();

    let dir_a = TempDir::new().expect("tempdir");
    let dir_b = TempDir::new().expect("tempdir");
    let path_a = dir_a
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();
    let path_b = dir_b
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    let slug = "filter-org".to_string();
    registry.organizations.insert(
        slug.clone(),
        Organization {
            name: "Filter Org".to_string(),
            description: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        },
    );
    registry.projects.insert(
        path_a.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: Some(slug.clone()),
            user_title: None,
        },
    );
    // path_b has no org
    registry.projects.insert(
        path_b.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: None,
            user_title: None,
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let opts = ListProjectsOptions {
        include_stale: true,
        include_uninitialized: true,
        include_temp: true, // project dirs are in temp
        organization_slug: Some(&slug),
        ..Default::default()
    };
    let projects = list_projects(opts).await.expect("Should succeed");
    assert!(projects.iter().any(|p| p.path == path_a));
    assert!(!projects.iter().any(|p| p.path == path_b));
}

#[tokio::test]
async fn test_list_projects_ungrouped_only() {
    let _lock = acquire_lock();

    let dir_a = TempDir::new().expect("tempdir");
    let dir_b = TempDir::new().expect("tempdir");
    let path_a = dir_a
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();
    let path_b = dir_b
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    let slug = "ungrouped-org".to_string();
    registry.organizations.insert(
        slug.clone(),
        Organization {
            name: "Ungrouped Org".to_string(),
            description: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        },
    );
    // path_a belongs to org
    registry.projects.insert(
        path_a.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: Some(slug.clone()),
            user_title: None,
        },
    );
    // path_b is ungrouped
    registry.projects.insert(
        path_b.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: None,
            user_title: None,
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let opts = ListProjectsOptions {
        include_stale: true,
        include_uninitialized: true,
        include_temp: true, // project dirs are in temp
        ungrouped_only: true,
        ..Default::default()
    };
    let projects = list_projects(opts).await.expect("Should succeed");
    assert!(!projects.iter().any(|p| p.path == path_a));
    assert!(projects.iter().any(|p| p.path == path_b));
}

#[tokio::test]
async fn test_list_projects_exclude_uninitialized() {
    let _lock = acquire_lock();

    // A temp dir that exists but has no .centy manifest = not initialized
    let dir = TempDir::new().expect("tempdir");
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    registry.projects.insert(
        path.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: None,
            user_title: None,
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    let opts = ListProjectsOptions {
        include_stale: true,
        include_uninitialized: false, // exclude uninitialized
        ..Default::default()
    };
    let projects = list_projects(opts).await.expect("Should succeed");
    // The project has no .centy manifest so it should be excluded
    assert!(!projects.iter().any(|p| p.path == path));
}

#[tokio::test]
async fn test_list_projects_exclude_temp() {
    let _lock = acquire_lock();

    // A path inside the temp directory (which is_ignored_path returns true)
    let temp_path = std::env::temp_dir()
        .join("centy-test-temp-project-list-unique-99")
        .to_string_lossy()
        .to_string();

    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    registry.projects.insert(
        temp_path.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: None,
            user_title: None,
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    // Without include_temp, temp-dir projects should be excluded
    let opts = ListProjectsOptions {
        include_stale: true,
        include_uninitialized: true,
        include_temp: false,
        ..Default::default()
    };
    let projects = list_projects(opts).await.expect("Should succeed");
    assert!(!projects.iter().any(|p| p.path == temp_path));
}

#[tokio::test]
async fn test_get_org_projects_empty() {
    let _lock = acquire_lock();

    let projects = get_org_projects("nonexistent-org-xyz-test", None)
        .await
        .expect("Should succeed");
    assert!(projects.is_empty());
}

#[tokio::test]
async fn test_get_org_projects_with_exclude() {
    let _lock = acquire_lock();

    // We use stale paths (non-existent) so include_stale=false in get_org_projects
    // will keep the list empty; we just test that exclude_path logic doesn't panic.
    let stale_path_a = "/stale/org-proj-a-unique".to_string();
    let stale_path_b = "/stale/org-proj-b-unique".to_string();

    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    let slug = "excl-org".to_string();
    registry.organizations.insert(
        slug.clone(),
        Organization {
            name: "Excl Org".to_string(),
            description: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        },
    );
    registry.projects.insert(
        stale_path_a.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: Some(slug.clone()),
            user_title: None,
        },
    );
    registry.projects.insert(
        stale_path_b.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: Some(slug.clone()),
            user_title: None,
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    // get_org_projects uses include_stale: false so stale paths won't appear,
    // but we verify exclude_path is wired in (no panic, correct result type)
    let projects = get_org_projects(&slug, Some(&stale_path_a))
        .await
        .expect("Should succeed");
    // Stale paths excluded by include_stale: false
    assert!(!projects.iter().any(|p| p.path == stale_path_a));
    assert!(!projects.iter().any(|p| p.path == stale_path_b));
}

// ──────────────────────────────────────────────────────────────────────────────
// enrich.rs: project with dangling org_slug (org not in registry)
// ──────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_projects_dangling_org_slug_triggers_sync() {
    let _lock = acquire_lock();

    // A temp dir that exists on disk (so include_stale:true includes it)
    let dir = TempDir::new().expect("tempdir");
    let path = dir
        .path()
        .canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();

    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read");
    // Insert project with an org_slug that does NOT exist in registry.organizations
    registry.projects.insert(
        path.clone(),
        TrackedProject {
            first_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            is_favorite: false,
            is_archived: false,
            organization_slug: Some("dangling-org-slug-xyz".to_string()),
            user_title: None,
        },
    );
    write_registry_unlocked(&registry).await.expect("write");
    drop(guard);

    // list_projects should handle the dangling org_slug gracefully (fires sync_org_from_project
    // via drop(), does not panic).
    let opts = ListProjectsOptions {
        include_stale: true,
        include_uninitialized: true,
        include_temp: true,
        ..Default::default()
    };
    let projects = list_projects(opts).await.expect("Should succeed");
    // The project should appear but with no org_name resolved
    let found = projects.iter().find(|p| p.path == path);
    assert!(found.is_some());
    assert!(found.unwrap().organization_name.is_none());
}

// ──────────────────────────────────────────────────────────────────────────────
// counts.rs: count_issues, count_md_files
// ──────────────────────────────────────────────────────────────────────────────

mod counts_tests {
    use super::{count_issues, count_md_files};
    use std::path::Path;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_count_issues_nonexistent_path() {
        let result = count_issues(Path::new("/nonexistent/issues/path/xyz123")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_count_issues_empty_dir() {
        let dir = TempDir::new().expect("tempdir");
        let result = count_issues(dir.path()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_count_issues_new_format_uuid_files() {
        let dir = TempDir::new().expect("tempdir");
        // Create UUID.md files (new format)
        let uuid1 = uuid::Uuid::new_v4().to_string();
        let uuid2 = uuid::Uuid::new_v4().to_string();
        std::fs::write(dir.path().join(format!("{uuid1}.md")), "content1").expect("write uuid1");
        std::fs::write(dir.path().join(format!("{uuid2}.md")), "content2").expect("write uuid2");
        // Also create a non-UUID file that should NOT be counted
        std::fs::write(dir.path().join("README.md"), "readme").expect("write readme");

        let result = count_issues(dir.path()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_count_issues_old_format_uuid_folders() {
        let dir = TempDir::new().expect("tempdir");
        // Create UUID directories (old format)
        let uuid1 = uuid::Uuid::new_v4().to_string();
        let uuid2 = uuid::Uuid::new_v4().to_string();
        std::fs::create_dir_all(dir.path().join(&uuid1)).expect("create dir uuid1");
        std::fs::create_dir_all(dir.path().join(&uuid2)).expect("create dir uuid2");
        // Non-UUID folder should NOT be counted
        std::fs::create_dir_all(dir.path().join("not-a-uuid")).expect("create regular dir");

        let result = count_issues(dir.path()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_count_issues_mixed_format() {
        let dir = TempDir::new().expect("tempdir");
        // One new format + one old format
        let uuid1 = uuid::Uuid::new_v4().to_string();
        let uuid2 = uuid::Uuid::new_v4().to_string();
        std::fs::write(dir.path().join(format!("{uuid1}.md")), "content").expect("write");
        std::fs::create_dir_all(dir.path().join(&uuid2)).expect("create dir");

        let result = count_issues(dir.path()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_count_md_files_nonexistent_path() {
        let result = count_md_files(Path::new("/nonexistent/docs/path/xyz456")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_count_md_files_empty_dir() {
        let dir = TempDir::new().expect("tempdir");
        let result = count_md_files(dir.path()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_count_md_files_with_md_files() {
        let dir = TempDir::new().expect("tempdir");
        std::fs::write(dir.path().join("doc1.md"), "doc1").expect("write doc1");
        std::fs::write(dir.path().join("doc2.md"), "doc2").expect("write doc2");
        std::fs::write(dir.path().join("readme.txt"), "txt").expect("write txt");

        let result = count_md_files(dir.path()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_count_md_files_skips_dirs() {
        let dir = TempDir::new().expect("tempdir");
        std::fs::write(dir.path().join("doc.md"), "content").expect("write");
        // A subdirectory should NOT be counted by count_md_files
        std::fs::create_dir_all(dir.path().join("subdir")).expect("create dir");

        let result = count_md_files(dir.path()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}
