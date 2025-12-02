mod common;

use centy_daemon::reconciliation::{
    build_reconciliation_plan, execute_reconciliation, ReconciliationDecisions,
};
use common::{create_test_dir, init_centy_project, verify_centy_structure};
use tokio::fs;

#[tokio::test]
async fn test_init_creates_centy_folder() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Verify structure
    verify_centy_structure(project_path);
}

#[tokio::test]
async fn test_init_creates_manifest_with_managed_files() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Read and verify manifest
    let manifest = centy_daemon::manifest::read_manifest(project_path)
        .await
        .expect("Should read manifest")
        .expect("Manifest should exist");

    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.centy_version, "0.1.0");
    assert!(!manifest.managed_files.is_empty());

    // Should have README.md and directories tracked
    let paths: Vec<&str> = manifest.managed_files.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"README.md"), "Should track README.md");
    assert!(paths.contains(&"issues/"), "Should track issues/");
    assert!(paths.contains(&"docs/"), "Should track docs/");
    assert!(paths.contains(&"assets/"), "Should track assets/");
}

#[tokio::test]
async fn test_init_idempotent() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize twice
    init_centy_project(project_path).await;
    init_centy_project(project_path).await;

    // Should still have valid structure
    verify_centy_structure(project_path);
}

#[tokio::test]
async fn test_reconciliation_plan_fresh_project() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Get plan for fresh project (no .centy folder)
    let plan = build_reconciliation_plan(project_path)
        .await
        .expect("Should build plan");

    // All managed files should be in to_create
    assert!(!plan.to_create.is_empty(), "Should have files to create");
    assert!(plan.to_restore.is_empty(), "Should have no files to restore");
    assert!(plan.to_reset.is_empty(), "Should have no files to reset");
    assert!(plan.up_to_date.is_empty(), "Should have no up-to-date files");
}

#[tokio::test]
async fn test_reconciliation_plan_initialized_project() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Get plan for initialized project
    let plan = build_reconciliation_plan(project_path)
        .await
        .expect("Should build plan");

    // No files to create, all should be up to date
    assert!(plan.to_create.is_empty(), "Should have no files to create");
    assert!(plan.to_restore.is_empty(), "Should have no files to restore");
    assert!(plan.to_reset.is_empty(), "Should have no files to reset");
    assert!(!plan.up_to_date.is_empty(), "Should have up-to-date files");
}

#[tokio::test]
async fn test_reconciliation_detects_deleted_files() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Delete a managed file
    let readme_path = project_path.join(".centy").join("README.md");
    fs::remove_file(&readme_path).await.expect("Should delete README");

    // Get plan - should detect deleted file
    let plan = build_reconciliation_plan(project_path)
        .await
        .expect("Should build plan");

    let restore_paths: Vec<&str> = plan.to_restore.iter().map(|f| f.path.as_str()).collect();
    assert!(
        restore_paths.contains(&"README.md"),
        "Should detect README.md needs restoration"
    );
}

#[tokio::test]
async fn test_reconciliation_detects_modified_files() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Modify a managed file
    let readme_path = project_path.join(".centy").join("README.md");
    fs::write(&readme_path, "Modified content")
        .await
        .expect("Should write");

    // Get plan - should detect modified file
    let plan = build_reconciliation_plan(project_path)
        .await
        .expect("Should build plan");

    let reset_paths: Vec<&str> = plan.to_reset.iter().map(|f| f.path.as_str()).collect();
    assert!(
        reset_paths.contains(&"README.md"),
        "Should detect README.md was modified"
    );
}

#[tokio::test]
async fn test_reconciliation_restore_with_decision() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Delete README
    let readme_path = project_path.join(".centy").join("README.md");
    fs::remove_file(&readme_path).await.expect("Should delete");
    assert!(!readme_path.exists());

    // Execute reconciliation with restore decision
    let mut decisions = ReconciliationDecisions::default();
    decisions.restore.insert("README.md".to_string());

    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should reconcile");

    // README should be restored
    assert!(readme_path.exists(), "README should be restored");
    assert!(
        result.restored.contains(&"README.md".to_string()),
        "Should report README as restored"
    );
}

#[tokio::test]
async fn test_reconciliation_force_mode() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Delete README
    let readme_path = project_path.join(".centy").join("README.md");
    fs::remove_file(&readme_path).await.expect("Should delete");

    // Execute with force=true (should restore without explicit decision)
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, true)
        .await
        .expect("Should reconcile");

    assert!(readme_path.exists(), "README should be restored in force mode");
    assert!(result.restored.contains(&"README.md".to_string()));
}

#[tokio::test]
async fn test_reconciliation_skip_without_decision() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Delete README
    let readme_path = project_path.join(".centy").join("README.md");
    fs::remove_file(&readme_path).await.expect("Should delete");

    // Execute without decision and force=false
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should reconcile");

    // README should NOT be restored (skipped)
    assert!(!readme_path.exists(), "README should remain deleted");
    assert!(
        result.skipped.contains(&"README.md".to_string()),
        "Should report README as skipped"
    );
}
