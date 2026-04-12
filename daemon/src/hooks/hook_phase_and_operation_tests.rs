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
fn test_hook_definition_yaml_serialization() {
    let hook = HookDefinition {
        pattern: "issue.creating".to_string(),
        command: "echo hello".to_string(),
        is_async: false,
        timeout: 30,
        enabled: true,
    };

    let yaml = serde_yaml::to_string(&hook).expect("Should serialize");
    let deserialized: HookDefinition = serde_yaml::from_str(&yaml).expect("Should deserialize");
    assert_eq!(deserialized.pattern, "issue.creating");
    assert_eq!(deserialized.command, "echo hello");
    assert!(!deserialized.is_async);
    assert_eq!(deserialized.timeout, 30);
    assert!(deserialized.enabled);
}

#[test]
fn test_hook_definition_defaults() {
    let yaml = "pattern: issue.creating\ncommand: echo test\n";
    let hook: HookDefinition = serde_yaml::from_str(yaml).expect("Should deserialize");
    assert!(!hook.is_async);
    assert_eq!(hook.timeout, 30);
    assert!(hook.enabled);
}

#[test]
fn test_hook_definition_async_rename() {
    let hook = HookDefinition {
        pattern: "test".to_string(),
        command: "cmd".to_string(),
        is_async: true,
        timeout: 60,
        enabled: false,
    };

    let yaml = serde_yaml::to_string(&hook).expect("Should serialize");
    assert!(yaml.contains("async:") || yaml.contains("async: "));
    assert!(!yaml.contains("is_async"));
}

#[test]
fn test_hooks_file_deserialization() {
    let yaml = "hooks:\n  - pattern: \"issue.creating\"\n    command: \"echo pre\"\n  - pattern: \"*.deleted\"\n    command: \"notify.sh\"\n    async: true\n    timeout: 10\n";
    let file: HooksFile = serde_yaml::from_str(yaml).expect("Should deserialize");
    assert_eq!(file.hooks.len(), 2);
    assert_eq!(file.hooks[0].pattern, "issue.creating");
    assert!(!file.hooks[0].is_async);
    assert_eq!(file.hooks[1].pattern, "*.deleted");
    assert!(file.hooks[1].is_async);
    assert_eq!(file.hooks[1].timeout, 10);
}

#[test]
fn test_hooks_file_empty_when_no_hooks_key() {
    let yaml = "";
    let file: HooksFile = serde_yaml::from_str(yaml).expect("Should deserialize");
    assert!(file.hooks.is_empty());
}
