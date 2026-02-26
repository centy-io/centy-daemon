use super::*;

#[test]
fn test_structured_error_json_format() {
    let se = StructuredError::new(
        "/tmp/project",
        "ITEM_NOT_FOUND",
        "Issue not found: abc".to_string(),
    );
    let json = se.to_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["cwd"], "/tmp/project");
    assert_eq!(parsed["messages"][0]["code"], "ITEM_NOT_FOUND");
    assert_eq!(parsed["messages"][0]["message"], "Issue not found: abc");
    assert!(parsed["messages"][0].get("tip").is_none());
}

#[test]
fn test_structured_error_with_tip() {
    let se = StructuredError::new(
        "/tmp/project",
        "NOT_INITIALIZED",
        "Project not initialized".to_string(),
    )
    .with_tip("Run 'centy init' to initialize the project");
    let json = se.to_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        parsed["messages"][0]["tip"],
        "Run 'centy init' to initialize the project"
    );
}

#[test]
fn test_structured_error_tip_skipped_when_none() {
    let se = StructuredError::new("/tmp/project", "IO_ERROR", "file not found".to_string());
    let json = se.to_json();
    assert!(!json.contains("\"tip\""));
}

#[test]
fn test_structured_error_logs_field() {
    let se = StructuredError::new("", "TEST", "test".to_string());
    let json = se.to_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    // logs field should exist (may be empty in test context since OnceLock not set)
    assert!(parsed.get("logs").is_some());
}
