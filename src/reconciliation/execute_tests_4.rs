use super::*;

#[tokio::test]
async fn test_execute_reconciliation_creates_hooks_yaml() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let result = execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
        .await
        .expect("Should execute reconciliation");
    let hooks_path = temp_dir.path().join(".centy").join("hooks.yaml");
    assert!(hooks_path.exists(), "hooks.yaml should be created");
    assert!(
        result.created.contains(&"hooks.yaml".to_string()),
        "Should report hooks.yaml as created"
    );
}

#[tokio::test]
async fn test_execute_reconciliation_hooks_yaml_content() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
        .await
        .expect("Should execute reconciliation");
    let hooks_path = temp_dir.path().join(".centy").join("hooks.yaml");
    let content = fs::read_to_string(&hooks_path)
        .await
        .expect("Should read hooks.yaml");
    assert!(
        content.contains("https://docs.centy.io/hooks"),
        "Should link to hooks documentation"
    );
    assert!(
        content.contains("issue.created"),
        "Should contain example event"
    );
    assert!(
        content.contains("$CENTY_ITEM_TITLE"),
        "Should contain example env var"
    );
}

#[tokio::test]
async fn test_execute_reconciliation_does_not_overwrite_existing_hooks_yaml() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
        .await
        .expect("Should execute first init");
    let hooks_path = temp_dir.path().join(".centy").join("hooks.yaml");
    fs::write(&hooks_path, "hooks:\n  - event: custom\n    run: echo hi\n")
        .await
        .expect("Should write custom hooks");
    let result = execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
        .await
        .expect("Should execute second init");
    let content = fs::read_to_string(&hooks_path)
        .await
        .expect("Should read");
    assert_eq!(
        content, "hooks:\n  - event: custom\n    run: echo hi\n",
        "Existing hooks.yaml should not be overwritten"
    );
    assert!(
        !result.created.contains(&"hooks.yaml".to_string()),
        "Should not re-create existing hooks.yaml"
    );
}
