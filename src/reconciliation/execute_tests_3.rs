use super::*;

#[tokio::test]
async fn test_execute_reconciliation_creates_issues_config_yaml() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let result = execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
        .await
        .expect("Should execute reconciliation");
    let config_path = temp_dir
        .path()
        .join(".centy")
        .join("issues")
        .join("config.yaml");
    assert!(config_path.exists(), "issues/config.yaml should be created");
    assert!(
        result.created.contains(&"issues/config.yaml".to_string()),
        "Should report issues/config.yaml as created"
    );
}

#[tokio::test]
async fn test_execute_reconciliation_creates_docs_config_yaml() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let result = execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
        .await
        .expect("Should execute reconciliation");
    let config_path = temp_dir
        .path()
        .join(".centy")
        .join("docs")
        .join("config.yaml");
    assert!(config_path.exists(), "docs/config.yaml should be created");
    assert!(
        result.created.contains(&"docs/config.yaml".to_string()),
        "Should report docs/config.yaml as created"
    );
}

#[tokio::test]
async fn test_execute_reconciliation_issues_config_yaml_content() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
        .await
        .expect("Should execute reconciliation");
    let config_path = temp_dir
        .path()
        .join(".centy")
        .join("issues")
        .join("config.yaml");
    let content = fs::read_to_string(&config_path)
        .await
        .expect("Should read config.yaml");
    assert!(content.contains("name: Issue"), "Should have name: Issue");
    assert!(
        content.contains("identifier: uuid"),
        "Should use uuid identifier"
    );
    assert!(
        content.contains("displayNumber: true"),
        "Should have displayNumber enabled"
    );
    assert!(
        content.contains("status: true"),
        "Should have status enabled"
    );
    assert!(
        content.contains("priority: true"),
        "Should have priority enabled"
    );
    assert!(
        content.contains("defaultStatus: open"),
        "Should have defaultStatus: open"
    );
}

#[tokio::test]
async fn test_execute_reconciliation_docs_config_yaml_content() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
        .await
        .expect("Should execute reconciliation");
    let config_path = temp_dir
        .path()
        .join(".centy")
        .join("docs")
        .join("config.yaml");
    let content = fs::read_to_string(&config_path)
        .await
        .expect("Should read config.yaml");
    assert!(content.contains("name: Doc"), "Should have name: Doc");
    assert!(
        content.contains("identifier: slug"),
        "Should use slug identifier"
    );
    assert!(
        content.contains("displayNumber: false"),
        "Docs should not have displayNumber"
    );
    assert!(
        content.contains("status: false"),
        "Docs should not have status"
    );
    assert!(
        content.contains("priority: false"),
        "Docs should not have priority"
    );
    assert!(
        !content.contains("defaultStatus:"),
        "Docs should not have defaultStatus"
    );
    assert!(
        !content.contains("priorityLevels:"),
        "Docs should not have priorityLevels"
    );
}

#[tokio::test]
async fn test_execute_reconciliation_does_not_overwrite_existing_config_yaml() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
        .await
        .expect("Should execute first init");
    let config_path = temp_dir
        .path()
        .join(".centy")
        .join("issues")
        .join("config.yaml");
    fs::write(&config_path, "name: CustomIssue\n")
        .await
        .expect("Should write custom config");
    let result = execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
        .await
        .expect("Should execute second init");
    let content = fs::read_to_string(&config_path).await.expect("Should read");
    assert_eq!(
        content, "name: CustomIssue\n",
        "Existing config.yaml should not be overwritten"
    );
    assert!(
        !result.created.contains(&"issues/config.yaml".to_string()),
        "Should not re-create existing issues/config.yaml"
    );
}

#[tokio::test]
async fn test_execute_reconciliation_resets_with_decision() {
    use tempfile::tempdir;
    use tokio::fs as async_fs;
    let temp_dir = tempdir().expect("Should create temp dir");
    execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
        .await
        .expect("Should initialize");
    let readme_path = temp_dir.path().join(".centy").join("README.md");
    async_fs::write(&readme_path, "Modified content")
        .await
        .expect("Should write");
    let mut decisions = ReconciliationDecisions::default();
    decisions.reset.insert("README.md".to_string());
    let result = execute_reconciliation(temp_dir.path(), decisions, false)
        .await
        .expect("Should execute");
    assert!(result.reset.contains(&"README.md".to_string()));
    let content = async_fs::read_to_string(&readme_path)
        .await
        .expect("Should read");
    assert!(content.contains("Centy Project"));
}
