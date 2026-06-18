//! Tests for `sync::sync_org_from_project`.
//! `super` here is `sync`, `super::super` is `organizations`.
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::await_holding_lock)]

use super::sync_org_from_project;
use crate::registry::storage::{get_lock, read_registry, write_registry_unlocked};
use crate::registry::types::{Organization, ProjectOrganization, TrackedProject};
use crate::utils::{get_centy_path, now_iso};
use tempfile::TempDir;

fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
    super::super::acquire_org_test_lock()
}

async fn write_org_file(project_path: &std::path::Path, org: &ProjectOrganization) {
    let centy = get_centy_path(project_path);
    tokio::fs::create_dir_all(&centy)
        .await
        .expect("create .centy");
    let path = centy.join("organization.json");
    let content = serde_json::to_string_pretty(org).expect("serialize");
    tokio::fs::write(path, content)
        .await
        .expect("write org file");
}

#[tokio::test]
async fn test_sync_org_no_org_file_returns_none() {
    let _lock = acquire_lock();

    let project_dir = TempDir::new().expect("project tmp");

    let result = sync_org_from_project(project_dir.path())
        .await
        .expect("should not error");

    assert!(result.is_none(), "expected None when no org file");
}

#[tokio::test]
async fn test_sync_org_auto_imports_new_org() {
    let _lock = acquire_lock();

    let project_dir = TempDir::new().expect("project tmp");

    let org_data = ProjectOrganization {
        slug: "sync-auto-imported-org".to_string(),
        name: "Sync Auto Imported Org".to_string(),
        description: Some("was auto imported".to_string()),
    };
    write_org_file(project_dir.path(), &org_data).await;

    let result = sync_org_from_project(project_dir.path())
        .await
        .expect("should succeed");

    let org = result.expect("should have auto-imported the org");
    assert_eq!(org.name, "Sync Auto Imported Org");
    assert_eq!(org.description, Some("was auto imported".to_string()));
}

#[tokio::test]
async fn test_sync_org_returns_existing_org_without_reimport() {
    let _lock = acquire_lock();

    // Pre-populate the registry with an org.
    {
        let guard = get_lock().lock().await;
        let mut registry = read_registry().await.expect("read");
        registry.organizations.insert(
            "sync-pre-existing".to_string(),
            Organization {
                name: "Sync Pre Existing".to_string(),
                description: None,
                created_at: now_iso(),
                updated_at: now_iso(),
            },
        );
        write_registry_unlocked(&registry).await.expect("write");
        drop(guard);
    }

    let project_dir = TempDir::new().expect("project tmp");
    let org_data = ProjectOrganization {
        slug: "sync-pre-existing".to_string(),
        name: "Sync Pre Existing".to_string(),
        description: None,
    };
    write_org_file(project_dir.path(), &org_data).await;

    let result = sync_org_from_project(project_dir.path())
        .await
        .expect("should succeed");

    let org = result.expect("should return existing org");
    assert_eq!(org.name, "Sync Pre Existing");
}

#[tokio::test]
async fn test_sync_org_links_project_when_tracked() {
    let _lock = acquire_lock();

    let project_dir = TempDir::new().expect("project tmp");

    // Register the project in the registry first.
    {
        let guard = get_lock().lock().await;
        let mut registry = read_registry().await.expect("read");
        let canonical = project_dir
            .path()
            .canonicalize()
            .unwrap_or_else(|_| project_dir.path().to_path_buf())
            .to_string_lossy()
            .to_string();
        registry.projects.insert(
            canonical,
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

    let org_data = ProjectOrganization {
        slug: "sync-link-org".to_string(),
        name: "Sync Link Org".to_string(),
        description: None,
    };
    write_org_file(project_dir.path(), &org_data).await;

    let result = sync_org_from_project(project_dir.path())
        .await
        .expect("should succeed");

    let org = result.expect("org");
    assert_eq!(org.name, "Sync Link Org");

    // Verify the project is now linked to the org.
    let registry = read_registry().await.expect("read after sync");
    let canonical = project_dir
        .path()
        .canonicalize()
        .unwrap_or_else(|_| project_dir.path().to_path_buf())
        .to_string_lossy()
        .to_string();
    if let Some(proj) = registry.projects.get(&canonical) {
        assert_eq!(proj.organization_slug.as_deref(), Some("sync-link-org"));
    }
}
