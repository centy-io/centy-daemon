#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::await_holding_lock)]

use super::org_repo::find_org_repo;
use super::storage::{
    acquire_registry_test_lock, get_lock, read_registry, write_registry_unlocked,
};
use super::types::{Organization, TrackedProject};
use crate::utils::now_iso;

async fn setup_registry_with_org_repo(project_path: &str, org_repo_path: &str, org_slug: &str) {
    let _guard = get_lock().lock().await;
    let mut registry = read_registry().await.unwrap();
    let now = now_iso();

    registry.organizations.insert(
        org_slug.to_string(),
        Organization {
            name: org_slug.to_string(),
            description: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        },
    );
    registry.projects.insert(
        project_path.to_string(),
        TrackedProject {
            first_accessed: now.clone(),
            last_accessed: now.clone(),
            is_favorite: false,
            is_archived: false,
            organization_slug: Some(org_slug.to_string()),
            user_title: None,
        },
    );
    registry.projects.insert(
        org_repo_path.to_string(),
        TrackedProject {
            first_accessed: now.clone(),
            last_accessed: now.clone(),
            is_favorite: false,
            is_archived: false,
            organization_slug: Some(org_slug.to_string()),
            user_title: None,
        },
    );
    registry.updated_at = now;
    write_registry_unlocked(&registry).await.unwrap();
}

#[tokio::test]
async fn test_find_org_repo_found() {
    let _lock = acquire_registry_test_lock();

    let project = "/home/user/myorg/my-project";
    let org_repo = "/home/user/myorg/.centy";
    setup_registry_with_org_repo(project, org_repo, "myorg").await;

    let result = find_org_repo(project).await.unwrap();
    assert_eq!(result, Some(org_repo.to_string()));
}

#[tokio::test]
async fn test_find_org_repo_not_found_no_centy_project() {
    let _lock = acquire_registry_test_lock();

    let project = "/home/user/anotherorg/my-project";
    // Register the project but no org repo
    {
        let _guard = get_lock().lock().await;
        let mut registry = read_registry().await.unwrap();
        let now = now_iso();
        registry.organizations.insert(
            "anotherorg".to_string(),
            Organization {
                name: "anotherorg".to_string(),
                description: None,
                created_at: now.clone(),
                updated_at: now.clone(),
            },
        );
        registry.projects.insert(
            project.to_string(),
            TrackedProject {
                first_accessed: now.clone(),
                last_accessed: now.clone(),
                is_favorite: false,
                is_archived: false,
                organization_slug: Some("anotherorg".to_string()),
                user_title: None,
            },
        );
        registry.updated_at = now;
        write_registry_unlocked(&registry).await.unwrap();
    }

    let result = find_org_repo(project).await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_find_org_repo_not_found_no_org() {
    let _lock = acquire_registry_test_lock();

    let project = "/home/user/noorg/my-project";
    // Register project with no org
    {
        let _guard = get_lock().lock().await;
        let mut registry = read_registry().await.unwrap();
        let now = now_iso();
        registry.projects.insert(
            project.to_string(),
            TrackedProject {
                first_accessed: now.clone(),
                last_accessed: now.clone(),
                is_favorite: false,
                is_archived: false,
                organization_slug: None,
                user_title: None,
            },
        );
        registry.updated_at = now;
        write_registry_unlocked(&registry).await.unwrap();
    }

    let result = find_org_repo(project).await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_find_org_repo_not_found_different_org() {
    let _lock = acquire_registry_test_lock();

    let project = "/home/user/org-a/my-project";
    let org_repo = "/home/user/org-b/.centy";
    // Project is in org-a, org repo is in org-b
    {
        let _guard = get_lock().lock().await;
        let mut registry = read_registry().await.unwrap();
        let now = now_iso();
        for slug in ["org-a-387", "org-b-387"] {
            registry.organizations.insert(
                slug.to_string(),
                Organization {
                    name: slug.to_string(),
                    description: None,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                },
            );
        }
        registry.projects.insert(
            project.to_string(),
            TrackedProject {
                first_accessed: now.clone(),
                last_accessed: now.clone(),
                is_favorite: false,
                is_archived: false,
                organization_slug: Some("org-a-387".to_string()),
                user_title: None,
            },
        );
        registry.projects.insert(
            org_repo.to_string(),
            TrackedProject {
                first_accessed: now.clone(),
                last_accessed: now.clone(),
                is_favorite: false,
                is_archived: false,
                organization_slug: Some("org-b-387".to_string()),
                user_title: None,
            },
        );
        registry.updated_at = now;
        write_registry_unlocked(&registry).await.unwrap();
    }

    let result = find_org_repo(project).await.unwrap();
    assert_eq!(
        result, None,
        "expected None when org repo is in a different org"
    );
}
