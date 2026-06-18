use super::*;

// ── StructuredError::with_tip on empty messages ───────────────────────────────

#[test]
fn test_with_tip_no_op_when_no_messages() {
    // Construct a StructuredError with empty messages to exercise the
    // `if let Some(msg) = self.messages.first_mut()` false branch.
    let empty_se = StructuredError {
        cwd: "/tmp".to_string(),
        logs: String::new(),
        messages: vec![],
    };
    // with_tip should be a no-op (not panic) when there are no messages
    let result_se = empty_se.with_tip("some tip");
    assert!(result_se.messages.is_empty());
}

// ── to_error_json ─────────────────────────────────────────────────────────────

#[test]
fn test_to_error_json_without_tip() {
    use crate::registry::RegistryError;
    let err = RegistryError::HomeDirNotFound;
    let json = to_error_json("/tmp/proj", &err);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["cwd"], "/tmp/proj");
    assert_eq!(parsed["messages"][0]["code"], "HOME_DIR_NOT_FOUND");
    // No tip for this variant
    assert!(parsed["messages"][0].get("tip").is_none());
}

#[test]
fn test_to_error_json_with_tip() {
    use crate::item::core::error::ItemError;
    let err = ItemError::NotInitialized;
    let json = to_error_json("/tmp/proj", &err);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["messages"][0]["code"], "NOT_INITIALIZED");
    assert!(parsed["messages"][0]["tip"].is_string());
    assert!(parsed["messages"][0]["tip"]
        .as_str()
        .unwrap()
        .contains("centy init"));
}

// ── StructuredError::new / with_tip / to_json ────────────────────────────────

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
