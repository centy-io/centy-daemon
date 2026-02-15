#![allow(clippy::indexing_slicing)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use centy_daemon::config::item_type_config::{read_item_type_config, write_item_type_config};
use centy_daemon::CentyConfig;
use centy_daemon::reconciliation::{execute_reconciliation, ReconciliationDecisions};
use common::create_test_dir;
use tokio::fs;

/// Fresh project: reconciliation creates both config.yaml files with defaults
#[tokio::test]
async fn test_fresh_project_creates_both_config_yaml() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should execute reconciliation");

    // Both config.yaml files should be reported as created
    assert!(
        result.created.contains(&"issues/config.yaml".to_string()),
        "Should create issues/config.yaml"
    );
    assert!(
        result.created.contains(&"docs/config.yaml".to_string()),
        "Should create docs/config.yaml"
    );

    // Files should exist on disk
    let centy_path = project_path.join(".centy");
    assert!(centy_path.join("issues").join("config.yaml").exists());
    assert!(centy_path.join("docs").join("config.yaml").exists());

    // Verify issues config content
    let issue_config = read_item_type_config(project_path, "issues")
        .await
        .expect("Should read")
        .expect("Should exist");
    assert_eq!(issue_config.name, "Issue");
    assert_eq!(issue_config.plural, "issues");
    assert_eq!(issue_config.identifier, "uuid");
    assert!(issue_config.features.status);
    assert!(issue_config.features.priority);

    // Verify docs config content
    let doc_config = read_item_type_config(project_path, "docs")
        .await
        .expect("Should read")
        .expect("Should exist");
    assert_eq!(doc_config.name, "Doc");
    assert_eq!(doc_config.plural, "docs");
    assert_eq!(doc_config.identifier, "slug");
    assert!(!doc_config.features.status);
    assert!(!doc_config.features.priority);
}

/// Custom config.json: pre-populate with custom allowedStates/priorityLevels,
/// verify mapping to statuses/defaultStatus/priorityLevels in issues/config.yaml
#[tokio::test]
async fn test_custom_config_json_maps_to_issues_config_yaml() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    let centy_path = project_path.join(".centy");

    // First init to create structure
    let decisions = ReconciliationDecisions::default();
    execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should init");

    // Remove the generated config.yaml files to simulate a pre-existing project
    fs::remove_file(centy_path.join("issues").join("config.yaml"))
        .await
        .expect("remove issues/config.yaml");
    fs::remove_file(centy_path.join("docs").join("config.yaml"))
        .await
        .expect("remove docs/config.yaml");

    // Write custom config.json
    let custom_config = r#"{
  "priorityLevels": 5,
  "customFields": [{"name": "env", "type": "string", "required": false}],
  "defaults": {},
  "allowedStates": ["open", "in-progress", "closed", "testing"],
  "defaultState": "open",
  "stateColors": {},
  "priorityColors": {},
  "hooks": []
}"#;
    fs::write(centy_path.join("config.json"), custom_config)
        .await
        .expect("write config");

    // Run reconciliation again — should recreate missing config.yaml files
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should reconcile");

    assert!(result.created.contains(&"issues/config.yaml".to_string()));

    // Verify issues config reflects custom config.json values
    let issue_config = read_item_type_config(project_path, "issues")
        .await
        .expect("Should read")
        .expect("Should exist");

    assert_eq!(
        issue_config.statuses,
        vec!["open", "in-progress", "closed", "testing"]
    );
    assert_eq!(issue_config.default_status, Some("open".to_string()));
    assert_eq!(issue_config.priority_levels, Some(5));
    assert_eq!(issue_config.custom_fields.len(), 1);
    assert_eq!(issue_config.custom_fields[0].name, "env");

    // Docs should have empty statuses (not mapped from config.json)
    let doc_config = read_item_type_config(project_path, "docs")
        .await
        .expect("Should read")
        .expect("Should exist");
    assert!(doc_config.statuses.is_empty());
    assert!(doc_config.default_status.is_none());
    assert!(doc_config.priority_levels.is_none());
}

/// Idempotent: run reconciliation twice, second run doesn't overwrite config.yaml
#[tokio::test]
async fn test_migration_idempotent() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // First run
    let decisions = ReconciliationDecisions::default();
    let result1 = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("First reconciliation");

    assert!(result1.created.contains(&"issues/config.yaml".to_string()));

    // Second run
    let decisions = ReconciliationDecisions::default();
    let result2 = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Second reconciliation");

    // config.yaml should NOT appear in created on second run
    assert!(
        !result2
            .created
            .contains(&"issues/config.yaml".to_string()),
        "issues/config.yaml should not be re-created"
    );
    assert!(
        !result2.created.contains(&"docs/config.yaml".to_string()),
        "docs/config.yaml should not be re-created"
    );
}

/// Preserves existing config.yaml: pre-create a custom config.yaml, run reconciliation, verify untouched
#[tokio::test]
async fn test_preserves_existing_config_yaml() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    let centy_path = project_path.join(".centy");

    // Create .centy structure manually
    fs::create_dir_all(centy_path.join("issues"))
        .await
        .expect("create issues/");
    fs::create_dir_all(centy_path.join("docs"))
        .await
        .expect("create docs/");

    // Pre-create a custom issues/config.yaml
    let custom_yaml = "name: CustomIssue\nplural: custom-issues\nidentifier: uuid\nfeatures:\n  displayNumber: true\n  status: true\n  priority: true\n  softDelete: true\n  assets: true\n  orgSync: true\n  move: true\n  duplicate: true\n";
    fs::write(centy_path.join("issues").join("config.yaml"), custom_yaml)
        .await
        .expect("write custom config.yaml");

    // Run reconciliation
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should reconcile");

    // issues/config.yaml should NOT be in created (it already existed)
    assert!(
        !result
            .created
            .contains(&"issues/config.yaml".to_string()),
        "Should not overwrite existing issues/config.yaml"
    );

    // docs/config.yaml should be created
    assert!(result.created.contains(&"docs/config.yaml".to_string()));

    // Verify custom content is preserved
    let content = fs::read_to_string(centy_path.join("issues").join("config.yaml"))
        .await
        .expect("read");
    assert_eq!(content, custom_yaml, "Custom config.yaml should be preserved");
}

/// Pre-existing project: simulate a real project (issues/, docs/, config.json
/// with custom states, some .md files), run reconciliation, verify config.yaml created
/// and existing files untouched
#[tokio::test]
async fn test_pre_existing_project() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    let centy_path = project_path.join(".centy");

    // Initialize then remove config.yaml files to simulate pre-existing project
    let decisions = ReconciliationDecisions::default();
    execute_reconciliation(project_path, decisions, false)
        .await
        .expect("initial setup");

    fs::remove_file(centy_path.join("issues").join("config.yaml"))
        .await
        .expect("remove issues/config.yaml");
    fs::remove_file(centy_path.join("docs").join("config.yaml"))
        .await
        .expect("remove docs/config.yaml");

    // Create some issue .md files (simulating existing data)
    let issue_content = "---\ntitle: Test Issue\nstatus: open\n---\nSome issue body\n";
    fs::write(
        centy_path.join("issues").join("test-issue.md"),
        issue_content,
    )
    .await
    .expect("write issue");

    let doc_content = "---\ntitle: Test Doc\n---\nSome doc body\n";
    fs::write(centy_path.join("docs").join("test-doc.md"), doc_content)
        .await
        .expect("write doc");

    // Write custom config.json with custom states
    let custom_config = r#"{
  "priorityLevels": 4,
  "customFields": [],
  "defaults": {},
  "allowedStates": ["open", "in-progress", "review", "closed"],
  "defaultState": "open",
  "stateColors": {},
  "priorityColors": {},
  "hooks": []
}"#;
    fs::write(centy_path.join("config.json"), custom_config)
        .await
        .expect("write config");

    // Run reconciliation
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(project_path, decisions, false)
        .await
        .expect("Should reconcile");

    // config.yaml files should be created
    assert!(result.created.contains(&"issues/config.yaml".to_string()));
    assert!(result.created.contains(&"docs/config.yaml".to_string()));

    // Existing .md files should be untouched
    let issue_read = fs::read_to_string(centy_path.join("issues").join("test-issue.md"))
        .await
        .expect("read issue");
    assert_eq!(issue_read, issue_content, "Issue file should be untouched");

    let doc_read = fs::read_to_string(centy_path.join("docs").join("test-doc.md"))
        .await
        .expect("read doc");
    assert_eq!(doc_read, doc_content, "Doc file should be untouched");

    // Verify issues config reflects the custom config.json states
    let issue_config = read_item_type_config(project_path, "issues")
        .await
        .expect("read config")
        .expect("config exists");
    assert_eq!(
        issue_config.statuses,
        vec!["open", "in-progress", "review", "closed"]
    );
    assert_eq!(issue_config.priority_levels, Some(4));
}

/// Verify that write_item_type_config + read_item_type_config roundtrips correctly
#[tokio::test]
async fn test_item_type_config_roundtrip_via_filesystem() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    let centy_path = project_path.join(".centy");
    fs::create_dir_all(centy_path.join("issues"))
        .await
        .expect("create dir");

    let mut config = CentyConfig::default();
    config.priority_levels = 7;
    config.allowed_states = vec!["open".to_string(), "done".to_string()];
    config.default_state = "open".to_string();

    let issue_config = centy_daemon::config::item_type_config::default_issue_config(&config);
    write_item_type_config(project_path, "issues", &issue_config)
        .await
        .expect("write");

    let read = read_item_type_config(project_path, "issues")
        .await
        .expect("read")
        .expect("exists");

    assert_eq!(read.name, "Issue");
    assert_eq!(read.priority_levels, Some(7));
    assert_eq!(read.statuses, vec!["open", "done"]);
    assert_eq!(read.default_status, Some("open".to_string()));
}
