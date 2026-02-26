use super::*;

// --- Phase tests ---

#[test]
fn test_phase_as_str() {
    assert_eq!(Phase::Pre.as_str(), "pre");
    assert_eq!(Phase::Post.as_str(), "post");
}

#[test]
fn test_phase_eq() {
    assert_eq!(Phase::Pre, Phase::Pre);
    assert_eq!(Phase::Post, Phase::Post);
    assert_ne!(Phase::Pre, Phase::Post);
}

// --- HookOperation tests ---

#[test]
fn test_hook_operation_as_str() {
    assert_eq!(HookOperation::Create.as_str(), "create");
    assert_eq!(HookOperation::Update.as_str(), "update");
    assert_eq!(HookOperation::Delete.as_str(), "delete");
    assert_eq!(HookOperation::SoftDelete.as_str(), "soft-delete");
    assert_eq!(HookOperation::Restore.as_str(), "restore");
    assert_eq!(HookOperation::Move.as_str(), "move");
    assert_eq!(HookOperation::Duplicate.as_str(), "duplicate");
}

// --- HookDefinition tests ---

#[test]
fn test_hook_definition_serialization() {
    let hook = HookDefinition {
        pattern: "pre:issue:create".to_string(),
        command: "echo hello".to_string(),
        is_async: false,
        timeout: 30,
        enabled: true,
    };

    let json = serde_json::to_string(&hook).expect("Should serialize");
    let deserialized: HookDefinition = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.pattern, "pre:issue:create");
    assert_eq!(deserialized.command, "echo hello");
    assert!(!deserialized.is_async);
    assert_eq!(deserialized.timeout, 30);
    assert!(deserialized.enabled);
}

#[test]
fn test_hook_definition_defaults() {
    let json = r#"{"pattern":"pre:issue:create","command":"echo test"}"#;
    let hook: HookDefinition = serde_json::from_str(json).expect("Should deserialize");
    assert!(!hook.is_async);
    assert_eq!(hook.timeout, 30);
    assert!(hook.enabled);
}

#[test]
fn test_hook_definition_camel_case() {
    let hook = HookDefinition {
        pattern: "test".to_string(),
        command: "cmd".to_string(),
        is_async: true,
        timeout: 60,
        enabled: false,
    };

    let json = serde_json::to_string(&hook).expect("Should serialize");
    assert!(json.contains("\"async\""));
    assert!(!json.contains("is_async"));
}

