//! Tests for `org_repo::find_org_repo`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::await_holding_lock)]

use super::find_org_repo;
use crate::registry::storage::{acquire_registry_test_lock, get_lock, read_registry, write_registry_unlocked};
use crate::registry::types::{Organization, TrackedProject};
use crate::utils::now_iso;

fn make_project(org_slug: Option<&str>) -> TrackedProject {
    TrackedProject {
        first_accessed: now_iso(),
        last_accessed: now_iso(),
        is_favorite: false,
        is_archived: false,
        organization_slug: org_slug.map(String::from),
        user_title: None,
    }
}

async fn setup_registry(projects: &[(&str, Option<&str>)], orgs: &[(&str, &str)]) {
    let guard = get_lock().lock().await;
    let mut registry = read_registry().await.expect("read registry");
    for (slug, name) in orgs {
        registry.organizations.insert(
            slug.to_string(),
            Organization {
                name: name.to_string(),
                description: None,
                created_at: now_iso(),
                updated_at: now_iso(),
            },
        );
    }
    for (path, org_slug) in projects {
        registry.projects.insert(path.to_string(), make_project(*org_slug));
    }
    write_registry_unlocked(&registry).await.expect("write registry");
    drop(guard);
}

#[tokio::test]
async fn test_find_org_repo_found() {
    let _lock = acquire_registry_test_lock();
    let project = "/tmp/test-387-project-found";
    let org_repo = "/tmp/test-387-org/.centy";
    setup_registry(
        &[(project, Some("org-387-found")), (org_repo, Some("org-387-found"))],
        &[("org-387-found", "Org 387 Found")],
    ).await;

    let result = find_org_repo(project).await.expect("find_org_repo");
    assert_eq!(result.as_deref(), Some(org_repo));
}

#[tokio::test]
async fn test_find_org_repo_not_found_no_org() {
    let _lock = acquire_registry_test_lock();
    let project = "/tmp/test-387-no-org-project";
    setup_registry(&[(project, None)], &[]).await;

    let result = find_org_repo(project).await.expect("find_org_repo");
    assert!(result.is_none(), "expected None when project has no org");
}

#[tokio::test]
async fn test_find_org_repo_not_found_no_centy_project() {
    let _lock = acquire_registry_test_lock();
    let project = "/tmp/test-387-lonely-project";
    let other = "/tmp/test-387-other-project";
    setup_registry(
        &[(project, Some("org-387-lonely")), (other, Some("org-387-lonely"))],
        &[("org-387-lonely", "Org 387 Lonely")],
    ).await;

    let result = find_org_repo(project).await.expect("find_org_repo");
    assert!(result.is_none(), "expected None when no project ends with /.centy");
}

#[tokio::test]
async fn test_find_org_repo_not_found_different_org() {
    let _lock = acquire_registry_test_lock();
    let project = "/tmp/test-387-org-a-project";
    let org_repo = "/tmp/test-387-org-b/.centy";
    setup_registry(
        &[(project, Some("org-387-a")), (org_repo, Some("org-387-b"))],
        &[("org-387-a", "Org A"), ("org-387-b", "Org B")],
    ).await;

    let result = find_org_repo(project).await.expect("find_org_repo");
    assert!(result.is_none(), "expected None when org repo is in a different org");
}
