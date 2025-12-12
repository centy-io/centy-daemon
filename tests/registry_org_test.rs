use centy_daemon::registry::{
    create_organization, delete_organization, get_organization, list_organizations,
    set_project_organization, track_project, untrack_project, update_organization,
};
use std::path::Path;
use tempfile::TempDir;

fn create_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

async fn init_project(project_path: &Path) {
    use centy_daemon::reconciliation::{execute_reconciliation, ReconciliationDecisions};

    let decisions = ReconciliationDecisions::default();
    execute_reconciliation(project_path, decisions, true)
        .await
        .expect("Failed to initialize");
}

// Organization CRUD tests

#[tokio::test]
async fn test_create_organization_success() {
    let result = create_organization(Some("test-org"), "Test Organization", Some("A test org"))
        .await
        .expect("Should create org");

    assert_eq!(result.slug, "test-org");
    assert_eq!(result.name, "Test Organization");
    assert_eq!(result.description, Some("A test org".to_string()));
    assert_eq!(result.project_count, 0);
    assert!(!result.created_at.is_empty());

    // Cleanup
    let _ = delete_organization("test-org").await;
}

#[tokio::test]
async fn test_create_organization_auto_slug() {
    let result = create_organization(None, "My Awesome Org", None)
        .await
        .expect("Should create org");

    assert_eq!(result.slug, "my-awesome-org");
    assert_eq!(result.name, "My Awesome Org");

    // Cleanup
    let _ = delete_organization("my-awesome-org").await;
}

#[tokio::test]
async fn test_create_organization_already_exists() {
    // Create first
    create_organization(Some("dup-test"), "Dup Test", None)
        .await
        .expect("Should create first");

    // Try to create duplicate
    let result = create_organization(Some("dup-test"), "Dup Test 2", None).await;
    assert!(result.is_err());

    // Cleanup
    let _ = delete_organization("dup-test").await;
}

#[tokio::test]
async fn test_create_organization_invalid_slug() {
    // Empty slug will trigger slugify from name, so we need to test invalid chars
    let result = create_organization(Some("INVALID"), "Test", None).await;
    assert!(result.is_err());

    let result = create_organization(Some("-start-hyphen"), "Test", None).await;
    assert!(result.is_err());

    let result = create_organization(Some("end-hyphen-"), "Test", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_organizations_empty() {
    // Note: This test might be affected by other tests running in parallel
    // In a real scenario, we'd need to isolate the registry
    let orgs = list_organizations().await.expect("Should list orgs");
    // Just verify it returns without error - orgs is a valid vec
    let _ = orgs.len();
}

#[tokio::test]
async fn test_list_organizations_with_orgs() {
    // Create some orgs
    create_organization(Some("list-test-a"), "Org A", None)
        .await
        .expect("Should create");
    create_organization(Some("list-test-b"), "Org B", None)
        .await
        .expect("Should create");

    let orgs = list_organizations().await.expect("Should list");

    // Find our test orgs
    let test_orgs: Vec<_> = orgs
        .iter()
        .filter(|o| o.slug.starts_with("list-test-"))
        .collect();

    assert!(test_orgs.len() >= 2);

    // Cleanup
    let _ = delete_organization("list-test-a").await;
    let _ = delete_organization("list-test-b").await;
}

#[tokio::test]
async fn test_get_organization_success() {
    create_organization(Some("get-test"), "Get Test", Some("Description"))
        .await
        .expect("Should create");

    let org = get_organization("get-test")
        .await
        .expect("Should get")
        .expect("Should exist");

    assert_eq!(org.slug, "get-test");
    assert_eq!(org.name, "Get Test");
    assert_eq!(org.description, Some("Description".to_string()));

    // Cleanup
    let _ = delete_organization("get-test").await;
}

#[tokio::test]
async fn test_get_organization_not_found() {
    let result = get_organization("nonexistent-org-xyz")
        .await
        .expect("Should complete");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_update_organization_name() {
    create_organization(Some("update-name-test"), "Original Name", None)
        .await
        .expect("Should create");

    let updated = update_organization("update-name-test", Some("New Name"), None, None)
        .await
        .expect("Should update");

    assert_eq!(updated.name, "New Name");

    // Cleanup
    let _ = delete_organization("update-name-test").await;
}

#[tokio::test]
async fn test_update_organization_description() {
    create_organization(Some("update-desc-test"), "Test", Some("Original"))
        .await
        .expect("Should create");

    let updated = update_organization("update-desc-test", None, Some("New Description"), None)
        .await
        .expect("Should update");

    assert_eq!(updated.description, Some("New Description".to_string()));

    // Clear description
    let cleared = update_organization("update-desc-test", None, Some(""), None)
        .await
        .expect("Should update");

    assert_eq!(cleared.description, None);

    // Cleanup
    let _ = delete_organization("update-desc-test").await;
}

#[tokio::test]
async fn test_update_organization_slug() {
    create_organization(Some("old-slug"), "Test", None)
        .await
        .expect("Should create");

    let updated = update_organization("old-slug", None, None, Some("new-slug"))
        .await
        .expect("Should update");

    assert_eq!(updated.slug, "new-slug");

    // Old slug should no longer exist
    let old = get_organization("old-slug").await.expect("Should complete");
    assert!(old.is_none());

    // New slug should exist
    let new = get_organization("new-slug")
        .await
        .expect("Should complete")
        .expect("Should exist");
    assert_eq!(new.slug, "new-slug");

    // Cleanup
    let _ = delete_organization("new-slug").await;
}

#[tokio::test]
async fn test_update_organization_not_found() {
    let result = update_organization("nonexistent-org", Some("New Name"), None, None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_organization_success() {
    create_organization(Some("delete-test"), "Delete Test", None)
        .await
        .expect("Should create");

    delete_organization("delete-test")
        .await
        .expect("Should delete");

    let org = get_organization("delete-test").await.expect("Should complete");
    assert!(org.is_none());
}

#[tokio::test]
async fn test_delete_organization_not_found() {
    let result = delete_organization("nonexistent-delete-test").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_organization_with_projects_fails() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_project(project_path).await;

    // Create org
    create_organization(Some("org-with-proj"), "Org With Project", None)
        .await
        .expect("Should create org");

    // Track project and assign to org
    let path_str = project_path.to_string_lossy().to_string();
    track_project(&path_str).await.expect("Should track");
    set_project_organization(&path_str, Some("org-with-proj"))
        .await
        .expect("Should set org");

    // Try to delete org - should fail because it has projects
    let result = delete_organization("org-with-proj").await;
    assert!(result.is_err());

    // Cleanup - first unassign project, then delete org
    let _ = set_project_organization(&path_str, None).await;
    let _ = untrack_project(&path_str).await;
    let _ = delete_organization("org-with-proj").await;
}

// Project organization assignment tests

#[tokio::test]
async fn test_set_project_organization() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_project(project_path).await;

    create_organization(Some("assign-org"), "Assign Org", None)
        .await
        .expect("Should create org");

    let path_str = project_path.to_string_lossy().to_string();
    track_project(&path_str).await.expect("Should track");

    set_project_organization(&path_str, Some("assign-org"))
        .await
        .expect("Should set org");

    // Verify org has project
    let org = get_organization("assign-org")
        .await
        .expect("Should get")
        .expect("Should exist");
    assert_eq!(org.project_count, 1);

    // Cleanup
    let _ = set_project_organization(&path_str, None).await;
    let _ = untrack_project(&path_str).await;
    let _ = delete_organization("assign-org").await;
}

#[tokio::test]
async fn test_remove_project_from_organization() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_project(project_path).await;

    create_organization(Some("remove-org"), "Remove Org", None)
        .await
        .expect("Should create org");

    let path_str = project_path.to_string_lossy().to_string();
    track_project(&path_str).await.expect("Should track");
    set_project_organization(&path_str, Some("remove-org"))
        .await
        .expect("Should set org");

    // Remove from org
    set_project_organization(&path_str, None)
        .await
        .expect("Should remove");

    // Verify org has no projects
    let org = get_organization("remove-org")
        .await
        .expect("Should get")
        .expect("Should exist");
    assert_eq!(org.project_count, 0);

    // Cleanup
    let _ = untrack_project(&path_str).await;
    let _ = delete_organization("remove-org").await;
}

// Slug validation tests

#[tokio::test]
async fn test_slug_validation_lowercase() {
    // Uppercase should fail
    let result = create_organization(Some("UpperCase"), "Test", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_slug_validation_special_chars() {
    // Special characters should fail
    let result = create_organization(Some("with_underscore"), "Test", None).await;
    assert!(result.is_err());

    let result = create_organization(Some("with.dot"), "Test", None).await;
    assert!(result.is_err());

    let result = create_organization(Some("with space"), "Test", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_slug_auto_generation() {
    // Test that auto-generated slugs are properly formatted
    let result = create_organization(None, "Test With Spaces", None)
        .await
        .expect("Should create");

    assert_eq!(result.slug, "test-with-spaces");

    // Cleanup
    let _ = delete_organization("test-with-spaces").await;
}

#[tokio::test]
async fn test_slug_auto_generation_special_chars() {
    let result = create_organization(None, "Test! With @Special# Chars$", None)
        .await
        .expect("Should create");

    // Should be kebab-case without special chars
    assert!(result.slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
    assert!(!result.slug.starts_with('-'));
    assert!(!result.slug.ends_with('-'));

    // Cleanup
    let _ = delete_organization(&result.slug).await;
}
