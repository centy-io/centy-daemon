use super::*;

#[test]
fn test_find_matching_hooks_empty() {
    let hooks: Vec<HookDefinition> = vec![];
    let result = find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Create);
    assert!(result.is_empty());
}

#[test]
fn test_find_matching_hooks_exact_match() {
    let hooks = vec![HookDefinition {
        pattern: "pre:issue:create".to_string(),
        command: "echo test".to_string(),
        is_async: false,
        timeout: 30,
        enabled: true,
    }];
    let result = find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Create);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_find_matching_hooks_no_match() {
    let hooks = vec![HookDefinition {
        pattern: "pre:issue:create".to_string(),
        command: "echo test".to_string(),
        is_async: false,
        timeout: 30,
        enabled: true,
    }];
    let result = find_matching_hooks(&hooks, Phase::Pre, "doc", HookOperation::Create);
    assert!(result.is_empty());
}

#[test]
fn test_find_matching_hooks_disabled_skipped() {
    let hooks = vec![HookDefinition {
        pattern: "pre:issue:create".to_string(),
        command: "echo test".to_string(),
        is_async: false,
        timeout: 30,
        enabled: false,
    }];
    let result = find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Create);
    assert!(result.is_empty());
}
