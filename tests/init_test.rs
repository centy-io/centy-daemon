#![allow(clippy::indexing_slicing)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use centy_daemon::config::item_type_config::read_item_type_config;
use centy_daemon::reconciliation::{
    build_reconciliation_plan, execute_reconciliation, ReconciliationDecisions,
};
use common::{create_test_dir, init_centy_project, verify_centy_structure};
use mdstore::IdStrategy;
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
    assert!(
        centy_path.join("archived").is_dir(),
        "Should create archived/"
    );
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
async fn test_init_creates_item_type_config_yaml() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    let centy_path = project_path.join(".centy");

    // All config.yaml files should be created
    assert!(
        centy_path.join("issues").join("config.yaml").exists(),
        "issues/config.yaml should be created on init"
    );
    assert!(
        centy_path.join("docs").join("config.yaml").exists(),
        "docs/config.yaml should be created on init"
    );
    assert!(
        centy_path.join("archived").join("config.yaml").exists(),
        "archived/config.yaml should be created on init"
    );
    assert!(
        result.created.contains(&"issues/config.yaml".to_string()),
        "Should report issues/config.yaml as created"
    );
    assert!(
        result.created.contains(&"docs/config.yaml".to_string()),
        "Should report docs/config.yaml as created"
    );
    assert!(
        result.created.contains(&"archived/config.yaml".to_string()),
        "Should report archived/config.yaml as created"
    );

    // Verify issues config has expected defaults
    let issues_content = fs::read_to_string(centy_path.join("issues").join("config.yaml"))
        .await
        .expect("Should read issues/config.yaml");
    assert!(
        issues_content.contains("displayNumber: true"),
        "issues/config.yaml should have displayNumber: true"
    );
    assert!(
        issues_content.contains("status: true"),
        "issues/config.yaml should have status: true"
    );
    assert!(
        issues_content.contains("priority: true"),
        "issues/config.yaml should have priority: true"
    );

    // Verify docs config has minimal features
    let docs_content = fs::read_to_string(centy_path.join("docs").join("config.yaml"))
        .await
        .expect("Should read docs/config.yaml");
    assert!(
        docs_content.contains("displayNumber: false"),
        "docs/config.yaml should have displayNumber: false"
    );
    assert!(
        docs_content.contains("status: false"),
        "docs/config.yaml should have status: false"
    );
    assert!(
        docs_content.contains("priority: false"),
        "docs/config.yaml should have priority: false"
    );
}

#[tokio::test]
async fn test_init_creates_archived_config_yaml() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    // archived/config.yaml should be reported as created
    assert!(
        result.created.contains(&"archived/config.yaml".to_string()),
        "Should create archived/config.yaml"
    );

    // File should exist on disk
    let config_path = project_path
        .join(".centy")
        .join("archived")
        .join("config.yaml");
    assert!(config_path.exists(), "archived/config.yaml should exist");

    // Verify archived config has expected defaults
    let config = read_item_type_config(project_path, "archived")
        .await
        .expect("Should read")
        .expect("Should exist");

    assert_eq!(config.name, "Archived");
    assert_eq!(config.identifier, IdStrategy::Uuid);
    assert!(!config.features.display_number);
    assert!(!config.features.status);
    assert!(!config.features.priority);
    assert!(config.features.assets);
    assert!(config.features.org_sync);
    assert!(config.features.move_item);
    assert!(!config.features.duplicate);
    assert_eq!(config.custom_fields.len(), 1);
    assert_eq!(config.custom_fields[0].name, "original_item_type");
}

#[tokio::test]
async fn test_init_does_not_overwrite_existing_item_type_config_yaml() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    let centy_path = project_path.join(".centy");

    // Create .centy/issues/ with a custom config.yaml
    fs::create_dir_all(centy_path.join("issues"))
        .await
        .expect("Should create issues dir");
    let custom_yaml = "name: CustomIssue\n";
    fs::write(centy_path.join("issues").join("config.yaml"), custom_yaml)
        .await
        .expect("Should write custom config.yaml");

    // Initialize
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    // issues/config.yaml should NOT be overwritten
    assert!(
        !result.created.contains(&"issues/config.yaml".to_string()),
        "Should not overwrite existing issues/config.yaml"
    );
    let content = fs::read_to_string(centy_path.join("issues").join("config.yaml"))
        .await
        .expect("Should read");
    assert_eq!(
        content, custom_yaml,
        "Existing issues/config.yaml should be preserved"
    );

    // docs/config.yaml should still be created
    assert!(
        result.created.contains(&"docs/config.yaml".to_string()),
        "Should create docs/config.yaml when missing"
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
    assert!(
        config_path.exists(),
        "config.json should be created on init"
    );
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
    assert_eq!(
        content, custom_config,
        "Existing config.json should not be overwritten"
    );
}

#[tokio::test]
async fn test_init_merges_cspell_json_preserving_custom_words() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Modify cspell.json with custom words
    let cspell_path = project_path.join(".centy").join("cspell.json");
    let custom_cspell = r#"{
  "version": "0.2",
  "language": "en",
  "words": [
    "centy",
    "customWord",
    "myProject"
  ],
  "ignorePaths": [
    ".centy-manifest.json",
    "dist/"
  ]
}"#;
    fs::write(&cspell_path, custom_cspell)
        .await
        .expect("Should write custom cspell.json");

    // Run init again (without explicit reset decision)
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    // cspell.json should be merged (in reset list), not skipped
    assert!(
        result.reset.contains(&"cspell.json".to_string()),
        "cspell.json should be merged (reported as reset)"
    );

    // Read the merged content
    let content = fs::read_to_string(&cspell_path)
        .await
        .expect("Should read cspell.json");
    let parsed: Value = serde_json::from_str(&content).expect("Should parse cspell.json");

    // Custom words should be preserved
    let words: Vec<&str> = parsed["words"]
        .as_array()
        .expect("words should be an array")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(
        words.contains(&"customWord"),
        "Custom word should be preserved"
    );
    assert!(
        words.contains(&"myProject"),
        "Custom word should be preserved"
    );

    // Template words should also be present
    assert!(words.contains(&"centy"), "Template word should be present");
    assert!(
        words.contains(&"displayNumber"),
        "Template word should be present"
    );

    // Custom ignorePaths should be preserved
    let paths: Vec<&str> = parsed["ignorePaths"]
        .as_array()
        .expect("ignorePaths should be an array")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(
        paths.contains(&"dist/"),
        "Custom ignorePath should be preserved"
    );
    assert!(
        paths.contains(&".centy-manifest.json"),
        "Template ignorePath should be present"
    );
}

#[tokio::test]
async fn test_init_merges_cspell_json_preserves_user_keys() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Modify cspell.json with extra top-level keys
    let cspell_path = project_path.join(".centy").join("cspell.json");
    let custom_cspell = r#"{
  "version": "0.2",
  "language": "en",
  "words": ["centy"],
  "ignorePaths": [".centy-manifest.json"],
  "flagWords": ["forbidden"],
  "dictionaries": ["custom-dict"]
}"#;
    fs::write(&cspell_path, custom_cspell)
        .await
        .expect("Should write custom cspell.json");

    // Run init again
    let decisions = ReconciliationDecisions::default();
    execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    // Read the merged content
    let content = fs::read_to_string(&cspell_path)
        .await
        .expect("Should read cspell.json");
    let parsed: Value = serde_json::from_str(&content).expect("Should parse cspell.json");

    // User-added keys should be preserved
    assert_eq!(parsed["flagWords"][0], "forbidden");
    assert_eq!(parsed["dictionaries"][0], "custom-dict");
}

#[tokio::test]
async fn test_init_merges_cspell_json_updates_version() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Modify cspell.json with an old version
    let cspell_path = project_path.join(".centy").join("cspell.json");
    let custom_cspell = r#"{
  "version": "0.1",
  "language": "fr",
  "words": ["centy", "custom"]
}"#;
    fs::write(&cspell_path, custom_cspell)
        .await
        .expect("Should write custom cspell.json");

    // Run init again
    let decisions = ReconciliationDecisions::default();
    execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    // Read the merged content
    let content = fs::read_to_string(&cspell_path)
        .await
        .expect("Should read cspell.json");
    let parsed: Value = serde_json::from_str(&content).expect("Should parse cspell.json");

    // version and language should be from the template
    assert_eq!(parsed["version"], "0.2");
    assert_eq!(parsed["language"], "en");

    // Custom word should still be there
    let words: Vec<&str> = parsed["words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(words.contains(&"custom"));
}

#[tokio::test]
async fn test_init_creates_cspell_json_from_template_when_missing() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize fresh project (cspell.json doesn't exist yet)
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    // cspell.json should be created from template
    assert!(
        result.created.contains(&"cspell.json".to_string()),
        "cspell.json should be created"
    );

    let cspell_path = project_path.join(".centy").join("cspell.json");
    assert!(cspell_path.exists(), "cspell.json should exist");
}

#[tokio::test]
async fn test_init_cspell_words_are_sorted_after_merge() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize
    init_centy_project(project_path).await;

    // Add unsorted custom words
    let cspell_path = project_path.join(".centy").join("cspell.json");
    let custom_cspell = r#"{
  "version": "0.2",
  "language": "en",
  "words": ["zebra", "apple", "centy"],
  "ignorePaths": [".centy-manifest.json"]
}"#;
    fs::write(&cspell_path, custom_cspell)
        .await
        .expect("Should write");

    // Run init again
    let decisions = ReconciliationDecisions::default();
    execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute");

    // Verify words are sorted
    let content = fs::read_to_string(&cspell_path).await.expect("Should read");
    let parsed: Value = serde_json::from_str(&content).expect("Should parse");

    let words: Vec<&str> = parsed["words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    let mut sorted_words = words.clone();
    sorted_words.sort_unstable();
    assert_eq!(words, sorted_words, "Words should be sorted after merge");
}

#[tokio::test]
async fn test_init_creates_issues_config_yaml() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    // issues/config.yaml should be reported as created
    assert!(
        result.created.contains(&"issues/config.yaml".to_string()),
        "Should create issues/config.yaml"
    );

    // File should exist on disk
    let config_path = project_path
        .join(".centy")
        .join("issues")
        .join("config.yaml");
    assert!(config_path.exists(), "issues/config.yaml should exist");

    // Verify default content
    let config = read_item_type_config(project_path, "issues")
        .await
        .expect("Should read")
        .expect("Should exist");

    assert_eq!(config.name, "Issue");
    assert_eq!(config.identifier, IdStrategy::Uuid);
    assert!(config.features.display_number);
    assert!(config.features.status);
    assert!(config.features.priority);
}

#[tokio::test]
async fn test_init_creates_docs_config_yaml() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    // docs/config.yaml should be reported as created
    assert!(
        result.created.contains(&"docs/config.yaml".to_string()),
        "Should create docs/config.yaml"
    );

    // File should exist on disk
    let config_path = project_path.join(".centy").join("docs").join("config.yaml");
    assert!(config_path.exists(), "docs/config.yaml should exist");

    // Verify minimal defaults (no displayNumber, status, or priority)
    let config = read_item_type_config(project_path, "docs")
        .await
        .expect("Should read")
        .expect("Should exist");

    assert_eq!(config.name, "Doc");
    assert_eq!(config.identifier, IdStrategy::Slug);
    assert!(!config.features.display_number);
    assert!(!config.features.status);
    assert!(!config.features.priority);
    assert!(config.statuses.is_empty());
    assert!(config.default_status.is_none());
    assert!(config.priority_levels.is_none());
}

#[tokio::test]
async fn test_init_does_not_overwrite_existing_issues_config_yaml() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    let centy_path = project_path.join(".centy");

    // Pre-create the issues directory with a custom config.yaml
    fs::create_dir_all(centy_path.join("issues"))
        .await
        .expect("Should create issues dir");
    let custom_yaml = "name: CustomIssue\nidentifier: uuid\nfeatures:\n  displayNumber: true\n  status: true\n  priority: true\n  assets: true\n  orgSync: true\n  move: true\n  duplicate: true\n";
    fs::write(centy_path.join("issues").join("config.yaml"), custom_yaml)
        .await
        .expect("Should write custom config.yaml");

    // Run init
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    // issues/config.yaml should NOT be in the created list
    assert!(
        !result.created.contains(&"issues/config.yaml".to_string()),
        "Should not overwrite existing issues/config.yaml"
    );

    // Existing content should be preserved
    let content = fs::read_to_string(centy_path.join("issues").join("config.yaml"))
        .await
        .expect("Should read");
    assert_eq!(
        content, custom_yaml,
        "Custom config.yaml should be preserved"
    );
}

#[tokio::test]
async fn test_init_config_yaml_idempotent() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // First init
    let decisions = ReconciliationDecisions::default();
    let result1 = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("First reconciliation");

    assert!(result1.created.contains(&"issues/config.yaml".to_string()));
    assert!(result1.created.contains(&"docs/config.yaml".to_string()));
    assert!(result1.created.contains(&"archived/config.yaml".to_string()));

    // Second init
    let result2 = execute_reconciliation(project_path, ReconciliationDecisions::default(), false)
        .await
        .expect("Second reconciliation");

    // config.yaml files should NOT appear in created on second run
    assert!(
        !result2.created.contains(&"issues/config.yaml".to_string()),
        "issues/config.yaml should not be re-created on second init"
    );
    assert!(
        !result2.created.contains(&"docs/config.yaml".to_string()),
        "docs/config.yaml should not be re-created on second init"
    );
    assert!(
        !result2.created.contains(&"archived/config.yaml".to_string()),
        "archived/config.yaml should not be re-created on second init"
    );
}
