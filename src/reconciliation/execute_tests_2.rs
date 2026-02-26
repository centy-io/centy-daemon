use super::*;

#[tokio::test]
async fn test_execute_reconciliation_creates_centy_folder() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let decisions = ReconciliationDecisions::default();
    execute_reconciliation(temp_dir.path(), decisions, false)
        .await
        .expect("Should execute reconciliation");
    assert!(temp_dir.path().join(".centy").exists());
}

#[tokio::test]
async fn test_execute_reconciliation_creates_managed_files() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(temp_dir.path(), decisions, false)
        .await
        .expect("Should execute reconciliation");
    assert!(!result.created.is_empty());
    let centy_path = temp_dir.path().join(".centy");
    assert!(centy_path.join("issues").is_dir());
    assert!(centy_path.join("docs").is_dir());
    assert!(centy_path.join("assets").is_dir());
    assert!(centy_path.join("templates").is_dir());
    assert!(centy_path.join("README.md").is_file());
    assert!(centy_path.join("cspell.json").is_file());
}

#[tokio::test]
async fn test_execute_reconciliation_writes_manifest() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(temp_dir.path(), decisions, false)
        .await
        .expect("Should execute reconciliation");
    let manifest_path = temp_dir.path().join(".centy").join(".centy-manifest.json");
    assert!(manifest_path.exists());
    assert_eq!(result.manifest.schema_version, 1);
    assert!(!result.manifest.centy_version.is_empty());
}

#[tokio::test]
async fn test_execute_reconciliation_idempotent() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let decisions = ReconciliationDecisions::default();
    let result1 = execute_reconciliation(temp_dir.path(), decisions.clone(), false)
        .await
        .expect("Should execute first time");
    let result2 =
        execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
            .await
            .expect("Should execute second time");
    assert!(!result1.created.is_empty());
    assert!(result2.created.is_empty());
}

#[tokio::test]
async fn test_execute_reconciliation_force_mode() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(temp_dir.path(), decisions, true)
        .await
        .expect("Should execute with force");
    assert!(!result.created.is_empty());
}

#[tokio::test]
async fn test_execute_reconciliation_skips_modified_without_decision() {
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
    let result = execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
        .await
        .expect("Should execute");
    assert!(result.skipped.contains(&"README.md".to_string()));
}

#[tokio::test]
async fn test_execute_reconciliation_creates_config_json() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let decisions = ReconciliationDecisions::default();
    let result = execute_reconciliation(temp_dir.path(), decisions, false)
        .await
        .expect("Should execute reconciliation");
    let config_path = temp_dir.path().join(".centy").join("config.json");
    assert!(config_path.exists(), "config.json should be created");
    assert!(
        result.created.contains(&"config.json".to_string()),
        "Should report config.json as created"
    );
    let content = fs::read_to_string(&config_path).await.expect("Should read");
    let value: serde_json::Value = serde_json::from_str(&content).expect("Should parse");
    assert!(
        value.as_object().unwrap().contains_key("hooks"),
        "config.json should contain hooks key"
    );
}
