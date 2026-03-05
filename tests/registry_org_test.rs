#![allow(clippy::indexing_slicing)]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::await_holding_lock)] // Intentional: serialize tests via std::sync::Mutex

use centy_daemon::registry::{
    create_organization, delete_organization, get_organization, list_organizations,
    set_project_organization, track_project, untrack_project, update_organization,
};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

// Tests share a global registry file. Serialize access to avoid concurrent
// read/write races that corrupt state.
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

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Generate a unique slug per test invocation to avoid collisions with stale
/// data left behind by previous (possibly crashed) test runs.
fn unique_slug(prefix: &str) -> String {
    let pid = std::process::id();
    let seq = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{pid}-{seq}")
}

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
    let _lock = acquire_lock();
    let slug = unique_slug("test-org");
    let result = create_organization(Some(&slug), "Test Organization", Some("A test org"))
        .await
        .expect("Should create org");

    assert_eq!(result.slug, slug);
    assert_eq!(result.name, "Test Organization");
    assert_eq!(result.description, Some("A test org".to_string()));
    assert_eq!(result.project_count, 0);
    assert!(!result.created_at.is_empty());

    // Cleanup
    let _ = delete_organization(&slug, false).await;
}

#[tokio::test]
async fn test_create_organization_auto_slug() {
    let _lock = acquire_lock();
    let suffix = unique_slug("auto");
    let name = format!("My Awesome {suffix}");
    let result = create_organization(None, &name, None)
        .await
        .expect("Should create org");

    assert_eq!(result.slug, format!("my-awesome-{suffix}"));
    assert_eq!(result.name, name);

    // Cleanup
    let _ = delete_organization(&result.slug, false).await;
}

#[tokio::test]
async fn test_create_organization_already_exists() {
    let _lock = acquire_lock();
    let slug = unique_slug("dup-test");
    // Create first
    create_organization(Some(&slug), "Dup Test", None)
        .await
        .expect("Should create first");

    // Try to create duplicate
    let result = create_organization(Some(&slug), "Dup Test 2", None).await;
    assert!(result.is_err());

    // Cleanup
    let _ = delete_organization(&slug, false).await;
}

#[tokio::test]
async fn test_create_organization_invalid_slug() {
    let _lock = acquire_lock();
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
    let _lock = acquire_lock();
    let orgs = list_organizations().await.expect("Should list orgs");
    // Just verify it returns without error - orgs is a valid vec
    let _ = orgs.len();
}

#[tokio::test]
async fn test_list_organizations_with_orgs() {
    let _lock = acquire_lock();
    // Create some orgs
    let slug_a = unique_slug("list-test-a");
    let slug_b = unique_slug("list-test-b");
    create_organization(Some(&slug_a), "Org A", None)
        .await
        .expect("Should create");
    create_organization(Some(&slug_b), "Org B", None)
        .await
        .expect("Should create");

    let orgs = list_organizations().await.expect("Should list");

    // Find our test orgs
    let has_a = orgs.iter().any(|o| o.slug == slug_a);
    let has_b = orgs.iter().any(|o| o.slug == slug_b);

    assert!(has_a && has_b);

    // Cleanup
    let _ = delete_organization(&slug_a, false).await;
    let _ = delete_organization(&slug_b, false).await;
}

#[tokio::test]
async fn test_get_organization_success() {
    let _lock = acquire_lock();
    let slug = unique_slug("get-test");
    create_organization(Some(&slug), "Get Test", Some("Description"))
        .await
        .expect("Should create");

    let org = get_organization(&slug)
        .await
        .expect("Should get")
        .expect("Should exist");

    assert_eq!(org.slug, slug);
    assert_eq!(org.name, "Get Test");
    assert_eq!(org.description, Some("Description".to_string()));

    // Cleanup
    let _ = delete_organization(&slug, false).await;
}

#[tokio::test]
async fn test_get_organization_not_found() {
    let _lock = acquire_lock();
    let result = get_organization("nonexistent-org-xyz")
        .await
        .expect("Should complete");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_update_organization_name() {
    let _lock = acquire_lock();
    let slug = unique_slug("update-name-test");
    create_organization(Some(&slug), "Original Name", None)
        .await
        .expect("Should create");

    let updated = update_organization(&slug, Some("New Name"), None, None)
        .await
        .expect("Should update");

    assert_eq!(updated.name, "New Name");

    // Cleanup
    let _ = delete_organization(&slug, false).await;
}

#[tokio::test]
async fn test_update_organization_description() {
    let _lock = acquire_lock();
    let slug = unique_slug("update-desc-test");
    create_organization(Some(&slug), "Test", Some("Original"))
        .await
        .expect("Should create");

    let updated = update_organization(&slug, None, Some("New Description"), None)
        .await
        .expect("Should update");

    assert_eq!(updated.description, Some("New Description".to_string()));

    // Clear description
    let cleared = update_organization(&slug, None, Some(""), None)
        .await
        .expect("Should update");

    assert_eq!(cleared.description, None);

    // Cleanup
    let _ = delete_organization(&slug, false).await;
}

#[tokio::test]
async fn test_update_organization_slug() {
    let _lock = acquire_lock();
    let old_slug = unique_slug("old-slug");
    let new_slug = unique_slug("new-slug");
    create_organization(Some(&old_slug), "Test", None)
        .await
        .expect("Should create");

    let updated = update_organization(&old_slug, None, None, Some(&new_slug))
        .await
        .expect("Should update");

    assert_eq!(updated.slug, new_slug);

    // Old slug should no longer exist
    let old = get_organization(&old_slug).await.expect("Should complete");
    assert!(old.is_none());

    // New slug should exist
    let new = get_organization(&new_slug)
        .await
        .expect("Should complete")
        .expect("Should exist");
    assert_eq!(new.slug, new_slug);

    // Cleanup
    let _ = delete_organization(&new_slug, false).await;
}

#[tokio::test]
async fn test_update_organization_not_found() {
    let _lock = acquire_lock();
    let result = update_organization("nonexistent-org", Some("New Name"), None, None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_organization_success() {
    let _lock = acquire_lock();
    let slug = unique_slug("delete-test");
    create_organization(Some(&slug), "Delete Test", None)
        .await
        .expect("Should create");

    delete_organization(&slug, false)
        .await
        .expect("Should delete");

    let org = get_organization(&slug).await.expect("Should complete");
    assert!(org.is_none());
}

#[tokio::test]
async fn test_delete_organization_not_found() {
    let _lock = acquire_lock();
    let result = delete_organization("nonexistent-delete-test", false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_organization_with_projects_fails() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_project(project_path).await;

    // Create org
    let slug = unique_slug("org-with-proj");
    create_organization(Some(&slug), "Org With Project", None)
        .await
        .expect("Should create org");

    // Track project and assign to org
    let path_str = project_path.to_string_lossy().to_string();
    track_project(&path_str).await.expect("Should track");
    set_project_organization(&path_str, Some(&slug))
        .await
        .expect("Should set org");

    // Try to delete org - should fail because it has projects
    let result = delete_organization(&slug, false).await;
    assert!(result.is_err());

    // Cleanup - first unassign project, then delete org
    let _ = set_project_organization(&path_str, None).await;
    let _ = untrack_project(&path_str).await;
    let _ = delete_organization(&slug, false).await;
}

// Project organization assignment tests

#[tokio::test]
async fn test_set_project_organization() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_project(project_path).await;

    let slug = unique_slug("assign-org");
    create_organization(Some(&slug), "Assign Org", None)
        .await
        .expect("Should create org");

    let path_str = project_path.to_string_lossy().to_string();
    track_project(&path_str).await.expect("Should track");

    set_project_organization(&path_str, Some(&slug))
        .await
        .expect("Should set org");

    // Verify org has project
    let org = get_organization(&slug)
        .await
        .expect("Should get")
        .expect("Should exist");
    assert_eq!(org.project_count, 1);

    // Cleanup
    let _ = set_project_organization(&path_str, None).await;
    let _ = untrack_project(&path_str).await;
    let _ = delete_organization(&slug, false).await;
}

#[tokio::test]
async fn test_remove_project_from_organization() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_project(project_path).await;

    let slug = unique_slug("remove-org");
    create_organization(Some(&slug), "Remove Org", None)
        .await
        .expect("Should create org");

    let path_str = project_path.to_string_lossy().to_string();
    track_project(&path_str).await.expect("Should track");
    set_project_organization(&path_str, Some(&slug))
        .await
        .expect("Should set org");

    // Remove from org
    set_project_organization(&path_str, None)
        .await
        .expect("Should remove");

    // Verify org has no projects
    let org = get_organization(&slug)
        .await
        .expect("Should get")
        .expect("Should exist");
    assert_eq!(org.project_count, 0);

    // Cleanup
    let _ = untrack_project(&path_str).await;
    let _ = delete_organization(&slug, false).await;
}

// Slug validation tests

#[tokio::test]
async fn test_slug_validation_lowercase() {
    let _lock = acquire_lock();
    // Uppercase should fail
    let result = create_organization(Some("UpperCase"), "Test", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_slug_validation_special_chars() {
    let _lock = acquire_lock();
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
    let _lock = acquire_lock();
    // Test that auto-generated slugs are properly formatted
    let suffix = unique_slug("slug");
    let name = format!("Test With {suffix}");
    let result = create_organization(None, &name, None)
        .await
        .expect("Should create");

    assert_eq!(result.slug, format!("test-with-{suffix}"));

    // Cleanup
    let _ = delete_organization(&result.slug, false).await;
}

#[tokio::test]
async fn test_slug_auto_generation_special_chars() {
    let _lock = acquire_lock();
    let suffix = unique_slug("sc");
    let name = format!("Test! With @Special# {suffix}$");
    let result = create_organization(None, &name, None)
        .await
        .expect("Should create");

    // Should be kebab-case without special chars
    assert!(result
        .slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
    assert!(!result.slug.starts_with('-'));
    assert!(!result.slug.ends_with('-'));

    // Cleanup
    let _ = delete_organization(&result.slug, false).await;
}

// Project name uniqueness tests

#[tokio::test]
async fn test_duplicate_project_name_in_same_org() {
    let _lock = acquire_lock();
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
    let slug = unique_slug("dup-test-org");
    create_organization(Some(&slug), "Dup Test Org", None)
        .await
        .expect("Should create org");

    let path1_str = project1_path.to_string_lossy().to_string();
    let path2_str = project2_path.to_string_lossy().to_string();

    // Track both projects
    track_project(&path1_str).await.expect("Should track 1");
    track_project(&path2_str).await.expect("Should track 2");

    // Assign first project to org - should succeed
    set_project_organization(&path1_str, Some(&slug))
        .await
        .expect("Should set org for first project");

    // Try to assign second project with same name to same org - should fail
    let result = set_project_organization(&path2_str, Some(&slug)).await;
    assert!(result.is_err(), "Should fail due to duplicate name");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("myapp"),
        "Error should mention project name"
    );
    assert!(error_msg.contains(&slug), "Error should mention org slug");

    // Cleanup
    let _ = set_project_organization(&path1_str, None).await;
    let _ = untrack_project(&path1_str).await;
    let _ = untrack_project(&path2_str).await;
    let _ = delete_organization(&slug, false).await;
}

#[tokio::test]
async fn test_duplicate_project_name_case_insensitive() {
    let _lock = acquire_lock();
    let temp_dir1 = create_test_dir();
    let temp_dir2 = create_test_dir();

    // Create projects with different case
    let project1_path = temp_dir1.path().join("MyApp");
    let project2_path = temp_dir2.path().join("myapp");

    std::fs::create_dir(&project1_path).expect("Should create dir 1");
    std::fs::create_dir(&project2_path).expect("Should create dir 2");

    init_project(&project1_path).await;
    init_project(&project2_path).await;

    let slug = unique_slug("case-test-org");
    create_organization(Some(&slug), "Case Test Org", None)
        .await
        .expect("Should create org");

    let path1_str = project1_path.to_string_lossy().to_string();
    let path2_str = project2_path.to_string_lossy().to_string();

    track_project(&path1_str).await.expect("Should track 1");
    track_project(&path2_str).await.expect("Should track 2");

    // Assign first project
    set_project_organization(&path1_str, Some(&slug))
        .await
        .expect("Should set org for first project");

    // Try to assign second project with different case - should fail
    let result = set_project_organization(&path2_str, Some(&slug)).await;
    assert!(result.is_err(), "Should fail due to case-insensitive match");

    // Cleanup
    let _ = set_project_organization(&path1_str, None).await;
    let _ = untrack_project(&path1_str).await;
    let _ = untrack_project(&path2_str).await;
    let _ = delete_organization(&slug, false).await;
}

#[tokio::test]
async fn test_reassign_same_project_idempotent() {
    let _lock = acquire_lock();
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_project(project_path).await;

    let slug = unique_slug("idempotent-org");
    create_organization(Some(&slug), "Idempotent Org", None)
        .await
        .expect("Should create org");

    let path_str = project_path.to_string_lossy().to_string();
    track_project(&path_str).await.expect("Should track");

    // Assign to org
    set_project_organization(&path_str, Some(&slug))
        .await
        .expect("Should set org");

    // Reassign same project to same org - should succeed (idempotent)
    let result = set_project_organization(&path_str, Some(&slug)).await;
    assert!(
        result.is_ok(),
        "Should succeed when reassigning same project"
    );

    // Cleanup
    let _ = set_project_organization(&path_str, None).await;
    let _ = untrack_project(&path_str).await;
    let _ = delete_organization(&slug, false).await;
}

#[tokio::test]
async fn test_same_name_in_different_orgs() {
    let _lock = acquire_lock();
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
    let slug_a = unique_slug("org-a");
    let slug_b = unique_slug("org-b");
    create_organization(Some(&slug_a), "Org A", None)
        .await
        .expect("Should create org A");
    create_organization(Some(&slug_b), "Org B", None)
        .await
        .expect("Should create org B");

    let path1_str = project1_path.to_string_lossy().to_string();
    let path2_str = project2_path.to_string_lossy().to_string();

    track_project(&path1_str).await.expect("Should track 1");
    track_project(&path2_str).await.expect("Should track 2");

    // Assign to different orgs - both should succeed
    set_project_organization(&path1_str, Some(&slug_a))
        .await
        .expect("Should set org A");
    set_project_organization(&path2_str, Some(&slug_b))
        .await
        .expect("Should set org B");

    // Cleanup
    let _ = set_project_organization(&path1_str, None).await;
    let _ = set_project_organization(&path2_str, None).await;
    let _ = untrack_project(&path1_str).await;
    let _ = untrack_project(&path2_str).await;
    let _ = delete_organization(&slug_a, false).await;
    let _ = delete_organization(&slug_b, false).await;
}

#[tokio::test]
async fn test_move_project_to_org_with_duplicate() {
    let _lock = acquire_lock();
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
    let source_slug = unique_slug("source-org");
    let target_slug = unique_slug("target-org");
    create_organization(Some(&source_slug), "Source Org", None)
        .await
        .expect("Should create source org");
    create_organization(Some(&target_slug), "Target Org", None)
        .await
        .expect("Should create target org");

    let path1_str = project1_path.to_string_lossy().to_string();
    let path2_str = project2_path.to_string_lossy().to_string();

    track_project(&path1_str).await.expect("Should track 1");
    track_project(&path2_str).await.expect("Should track 2");

    // Assign projects to different orgs
    set_project_organization(&path1_str, Some(&source_slug))
        .await
        .expect("Should set source org");
    set_project_organization(&path2_str, Some(&target_slug))
        .await
        .expect("Should set target org");

    // Try to move project1 to target-org where myapp already exists - should fail
    let result = set_project_organization(&path1_str, Some(&target_slug)).await;
    assert!(
        result.is_err(),
        "Should fail when moving to org with duplicate"
    );

    // Cleanup
    let _ = set_project_organization(&path1_str, None).await;
    let _ = set_project_organization(&path2_str, None).await;
    let _ = untrack_project(&path1_str).await;
    let _ = untrack_project(&path2_str).await;
    let _ = delete_organization(&source_slug, false).await;
    let _ = delete_organization(&target_slug, false).await;
}

#[tokio::test]
async fn test_unorganized_projects_allow_duplicates() {
    let _lock = acquire_lock();
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
    let _lock = acquire_lock();
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
    let slug1 = unique_slug("org-remove-1");
    let slug2 = unique_slug("org-remove-2");
    create_organization(Some(&slug1), "Org 1", None)
        .await
        .expect("Should create org 1");
    create_organization(Some(&slug2), "Org 2", None)
        .await
        .expect("Should create org 2");

    let path1_str = project1_path.to_string_lossy().to_string();
    let path2_str = project2_path.to_string_lossy().to_string();

    track_project(&path1_str).await.expect("Should track 1");
    track_project(&path2_str).await.expect("Should track 2");

    // Assign to different orgs
    set_project_organization(&path1_str, Some(&slug1))
        .await
        .expect("Should set org 1");
    set_project_organization(&path2_str, Some(&slug2))
        .await
        .expect("Should set org 2");

    // Remove project1 from its org - should succeed even though myapp exists in org 2
    let result = set_project_organization(&path1_str, None).await;
    assert!(result.is_ok(), "Should succeed removing from org");

    // Cleanup
    let _ = set_project_organization(&path2_str, None).await;
    let _ = untrack_project(&path1_str).await;
    let _ = untrack_project(&path2_str).await;
    let _ = delete_organization(&slug1, false).await;
    let _ = delete_organization(&slug2, false).await;
}
