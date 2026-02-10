#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use centy_daemon::reconciliation::{
    build_reconciliation_plan, execute_reconciliation, ReconciliationDecisions,
};
use common::{create_test_dir, init_centy_project, verify_centy_structure};
use serde_json::Value;
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
async fn test_init_creates_manifest_and_structure() {
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
    assert_eq!(manifest.centy_version, centy_daemon::utils::CENTY_VERSION);

    // Verify files and directories exist on disk
    let centy_path = project_path.join(".centy");
    assert!(
        centy_path.join("README.md").exists(),
        "Should create README.md"
    );
    assert!(centy_path.join("issues").is_dir(), "Should create issues/");
    assert!(centy_path.join("docs").is_dir(), "Should create docs/");
    assert!(centy_path.join("assets").is_dir(), "Should create assets/");
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
    assert!(
        plan.to_restore.is_empty(),
        "Should have no files to restore"
    );
    assert!(plan.to_reset.is_empty(), "Should have no files to reset");
    assert!(
        plan.up_to_date.is_empty(),
        "Should have no up-to-date files"
    );
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
    assert!(
        plan.to_restore.is_empty(),
        "Should have no files to restore"
    );
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
    fs::remove_file(&readme_path)
        .await
        .expect("Should delete README");

    // Get plan - should detect deleted file needs creation
    let plan = build_reconciliation_plan(project_path)
        .await
        .expect("Should build plan");

    // Without manifest-based file tracking, deleted files appear in to_create
    let create_paths: Vec<&str> = plan.to_create.iter().map(|f| f.path.as_str()).collect();
    assert!(
        create_paths.contains(&"README.md"),
        "Should detect README.md needs to be created"
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
async fn test_reconciliation_recreates_deleted_file() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Delete README
    let readme_path = project_path.join(".centy").join("README.md");
    fs::remove_file(&readme_path).await.expect("Should delete");
    assert!(!readme_path.exists());

    // Execute reconciliation - deleted files are treated as new and created
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should reconcile");

    // README should be recreated (in the created list, not restored)
    assert!(readme_path.exists(), "README should be recreated");
    assert!(
        result.created.contains(&"README.md".to_string()),
        "Should report README as created"
    );
}

#[tokio::test]
async fn test_reconciliation_force_mode_recreates_missing() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Delete README
    let readme_path = project_path.join(".centy").join("README.md");
    fs::remove_file(&readme_path).await.expect("Should delete");

    // Execute with force=true - missing files are created
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, true)
        .await
        .expect("Should reconcile");

    assert!(
        readme_path.exists(),
        "README should be recreated in force mode"
    );
    assert!(result.created.contains(&"README.md".to_string()));
}

#[tokio::test]
async fn test_reconciliation_skip_modified_without_decision() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Modify README (not delete - modified files require decision to reset)
    let readme_path = project_path.join(".centy").join("README.md");
    fs::write(&readme_path, "Modified content by user")
        .await
        .expect("Should write");

    // Execute without reset decision
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should reconcile");

    // README should remain modified (skipped reset)
    let content = fs::read_to_string(&readme_path).await.expect("Should read");
    assert_eq!(
        content, "Modified content by user",
        "README should remain modified"
    );
    assert!(
        result.skipped.contains(&"README.md".to_string()),
        "Should report README as skipped"
    );
}

#[tokio::test]
async fn test_init_creates_config_json_with_hooks() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    // config.json should be created
    let config_path = project_path.join(".centy").join("config.json");
    assert!(config_path.exists(), "config.json should be created on init");
    assert!(
        result.created.contains(&"config.json".to_string()),
        "Should report config.json as created"
    );

    // Verify it contains the hooks property
    let content = fs::read_to_string(&config_path)
        .await
        .expect("Should read config.json");
    let value: Value = serde_json::from_str(&content).expect("Should parse config.json");
    let obj = value.as_object().expect("Config should be an object");

    assert!(
        obj.contains_key("hooks"),
        "config.json should contain hooks property"
    );
    assert_eq!(
        obj.get("hooks"),
        Some(&Value::Array(vec![])),
        "hooks should be an empty array"
    );
}

#[tokio::test]
async fn test_init_does_not_overwrite_existing_config_json() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Create .centy dir and a pre-existing config.json
    let centy_path = project_path.join(".centy");
    fs::create_dir_all(&centy_path)
        .await
        .expect("Should create .centy dir");
    let custom_config = r#"{"priorityLevels": 5, "hooks": []}"#;
    fs::write(centy_path.join("config.json"), custom_config)
        .await
        .expect("Should write config");

    // Initialize
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    // config.json should NOT be in the created list
    assert!(
        !result.created.contains(&"config.json".to_string()),
        "Should not re-create existing config.json"
    );

    // Content should remain unchanged
    let content = fs::read_to_string(centy_path.join("config.json"))
        .await
        .expect("Should read config.json");
    assert_eq!(content, custom_config, "Existing config.json should not be overwritten");
}
