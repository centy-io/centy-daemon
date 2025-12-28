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

// Project name uniqueness tests

#[tokio::test]
async fn test_duplicate_project_name_in_same_org() {
    let temp_dir1 = create_test_dir();
    let temp_dir2 = create_test_dir();

    // Create two projects with same directory name but different parents
    let project1_path = temp_dir1.path().join("myapp");
    let project2_path = temp_dir2.path().join("myapp");

    std::fs::create_dir(&project1_path).expect("Should create dir 1");
    std::fs::create_dir(&project2_path).expect("Should create dir 2");

    init_project(&project1_path).await;
    init_project(&project2_path).await;

    // Create org
    create_organization(Some("dup-test-org"), "Dup Test Org", None)
        .await
        .expect("Should create org");

    let path1_str = project1_path.to_string_lossy().to_string();
    let path2_str = project2_path.to_string_lossy().to_string();

    // Track both projects
    track_project(&path1_str).await.expect("Should track 1");
    track_project(&path2_str).await.expect("Should track 2");

    // Assign first project to org - should succeed
    set_project_organization(&path1_str, Some("dup-test-org"))
        .await
        .expect("Should set org for first project");

    // Try to assign second project with same name to same org - should fail
    let result = set_project_organization(&path2_str, Some("dup-test-org")).await;
    assert!(result.is_err(), "Should fail due to duplicate name");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("myapp"), "Error should mention project name");
    assert!(error_msg.contains("dup-test-org"), "Error should mention org slug");

    // Cleanup
    let _ = set_project_organization(&path1_str, None).await;
    let _ = untrack_project(&path1_str).await;
    let _ = untrack_project(&path2_str).await;
    let _ = delete_organization("dup-test-org").await;
}

#[tokio::test]
async fn test_duplicate_project_name_case_insensitive() {
    let temp_dir1 = create_test_dir();
    let temp_dir2 = create_test_dir();

    // Create projects with different case
    let project1_path = temp_dir1.path().join("MyApp");
    let project2_path = temp_dir2.path().join("myapp");

    std::fs::create_dir(&project1_path).expect("Should create dir 1");
    std::fs::create_dir(&project2_path).expect("Should create dir 2");

    init_project(&project1_path).await;
    init_project(&project2_path).await;

    create_organization(Some("case-test-org"), "Case Test Org", None)
        .await
        .expect("Should create org");

    let path1_str = project1_path.to_string_lossy().to_string();
    let path2_str = project2_path.to_string_lossy().to_string();

    track_project(&path1_str).await.expect("Should track 1");
    track_project(&path2_str).await.expect("Should track 2");

    // Assign first project
    set_project_organization(&path1_str, Some("case-test-org"))
        .await
        .expect("Should set org for first project");

    // Try to assign second project with different case - should fail
    let result = set_project_organization(&path2_str, Some("case-test-org")).await;
    assert!(result.is_err(), "Should fail due to case-insensitive match");

    // Cleanup
    let _ = set_project_organization(&path1_str, None).await;
    let _ = untrack_project(&path1_str).await;
    let _ = untrack_project(&path2_str).await;
    let _ = delete_organization("case-test-org").await;
}

#[tokio::test]
async fn test_reassign_same_project_idempotent() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_project(project_path).await;

    create_organization(Some("idempotent-org"), "Idempotent Org", None)
        .await
        .expect("Should create org");

    let path_str = project_path.to_string_lossy().to_string();
    track_project(&path_str).await.expect("Should track");

    // Assign to org
    set_project_organization(&path_str, Some("idempotent-org"))
        .await
        .expect("Should set org");

    // Reassign same project to same org - should succeed (idempotent)
    let result = set_project_organization(&path_str, Some("idempotent-org")).await;
    assert!(result.is_ok(), "Should succeed when reassigning same project");

    // Cleanup
    let _ = set_project_organization(&path_str, None).await;
    let _ = untrack_project(&path_str).await;
    let _ = delete_organization("idempotent-org").await;
}

#[tokio::test]
async fn test_same_name_in_different_orgs() {
    let temp_dir1 = create_test_dir();
    let temp_dir2 = create_test_dir();

    // Create projects with same name
    let project1_path = temp_dir1.path().join("myapp");
    let project2_path = temp_dir2.path().join("myapp");

    std::fs::create_dir(&project1_path).expect("Should create dir 1");
    std::fs::create_dir(&project2_path).expect("Should create dir 2");

    init_project(&project1_path).await;
    init_project(&project2_path).await;

    // Create two orgs
    create_organization(Some("org-a"), "Org A", None)
        .await
        .expect("Should create org A");
    create_organization(Some("org-b"), "Org B", None)
        .await
        .expect("Should create org B");

    let path1_str = project1_path.to_string_lossy().to_string();
    let path2_str = project2_path.to_string_lossy().to_string();

    track_project(&path1_str).await.expect("Should track 1");
    track_project(&path2_str).await.expect("Should track 2");

    // Assign to different orgs - both should succeed
    set_project_organization(&path1_str, Some("org-a"))
        .await
        .expect("Should set org A");
    set_project_organization(&path2_str, Some("org-b"))
        .await
        .expect("Should set org B");

    // Cleanup
    let _ = set_project_organization(&path1_str, None).await;
    let _ = set_project_organization(&path2_str, None).await;
    let _ = untrack_project(&path1_str).await;
    let _ = untrack_project(&path2_str).await;
    let _ = delete_organization("org-a").await;
    let _ = delete_organization("org-b").await;
}

#[tokio::test]
async fn test_move_project_to_org_with_duplicate() {
    let temp_dir1 = create_test_dir();
    let temp_dir2 = create_test_dir();

    // Create projects with same name
    let project1_path = temp_dir1.path().join("myapp");
    let project2_path = temp_dir2.path().join("myapp");

    std::fs::create_dir(&project1_path).expect("Should create dir 1");
    std::fs::create_dir(&project2_path).expect("Should create dir 2");

    init_project(&project1_path).await;
    init_project(&project2_path).await;

    // Create two orgs
    create_organization(Some("source-org"), "Source Org", None)
        .await
        .expect("Should create source org");
    create_organization(Some("target-org"), "Target Org", None)
        .await
        .expect("Should create target org");

    let path1_str = project1_path.to_string_lossy().to_string();
    let path2_str = project2_path.to_string_lossy().to_string();

    track_project(&path1_str).await.expect("Should track 1");
    track_project(&path2_str).await.expect("Should track 2");

    // Assign projects to different orgs
    set_project_organization(&path1_str, Some("source-org"))
        .await
        .expect("Should set source org");
    set_project_organization(&path2_str, Some("target-org"))
        .await
        .expect("Should set target org");

    // Try to move project1 to target-org where myapp already exists - should fail
    let result = set_project_organization(&path1_str, Some("target-org")).await;
    assert!(result.is_err(), "Should fail when moving to org with duplicate");

    // Cleanup
    let _ = set_project_organization(&path1_str, None).await;
    let _ = set_project_organization(&path2_str, None).await;
    let _ = untrack_project(&path1_str).await;
    let _ = untrack_project(&path2_str).await;
    let _ = delete_organization("source-org").await;
    let _ = delete_organization("target-org").await;
}

#[tokio::test]
async fn test_unorganized_projects_allow_duplicates() {
    let temp_dir1 = create_test_dir();
    let temp_dir2 = create_test_dir();

    // Create projects with same name
    let project1_path = temp_dir1.path().join("myapp");
    let project2_path = temp_dir2.path().join("myapp");

    std::fs::create_dir(&project1_path).expect("Should create dir 1");
    std::fs::create_dir(&project2_path).expect("Should create dir 2");

    init_project(&project1_path).await;
    init_project(&project2_path).await;

    let path1_str = project1_path.to_string_lossy().to_string();
    let path2_str = project2_path.to_string_lossy().to_string();

    // Track both without assigning to org - should succeed
    track_project(&path1_str).await.expect("Should track 1");
    track_project(&path2_str).await.expect("Should track 2");

    // Both projects exist without org, both named "myapp" - this is allowed
    // No need to assert anything specific, just that tracking succeeded

    // Cleanup
    let _ = untrack_project(&path1_str).await;
    let _ = untrack_project(&path2_str).await;
}

#[tokio::test]
async fn test_remove_from_org_with_duplicate_elsewhere() {
    let temp_dir1 = create_test_dir();
    let temp_dir2 = create_test_dir();

    // Create projects with same name
    let project1_path = temp_dir1.path().join("myapp");
    let project2_path = temp_dir2.path().join("myapp");

    std::fs::create_dir(&project1_path).expect("Should create dir 1");
    std::fs::create_dir(&project2_path).expect("Should create dir 2");

    init_project(&project1_path).await;
    init_project(&project2_path).await;

    // Create two orgs
    create_organization(Some("org-remove-1"), "Org 1", None)
        .await
        .expect("Should create org 1");
    create_organization(Some("org-remove-2"), "Org 2", None)
        .await
        .expect("Should create org 2");

    let path1_str = project1_path.to_string_lossy().to_string();
    let path2_str = project2_path.to_string_lossy().to_string();

    track_project(&path1_str).await.expect("Should track 1");
    track_project(&path2_str).await.expect("Should track 2");

    // Assign to different orgs
    set_project_organization(&path1_str, Some("org-remove-1"))
        .await
        .expect("Should set org 1");
    set_project_organization(&path2_str, Some("org-remove-2"))
        .await
        .expect("Should set org 2");

    // Remove project1 from its org - should succeed even though myapp exists in org 2
    let result = set_project_organization(&path1_str, None).await;
    assert!(result.is_ok(), "Should succeed removing from org");

    // Cleanup
    let _ = set_project_organization(&path2_str, None).await;
    let _ = untrack_project(&path1_str).await;
    let _ = untrack_project(&path2_str).await;
    let _ = delete_organization("org-remove-1").await;
    let _ = delete_organization("org-remove-2").await;
}
