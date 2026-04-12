use super::*;

#[test]
fn test_find_matching_hooks_specificity_order() {
    let hooks = vec![
        HookDefinition {
            pattern: "*.*".to_string(),
            command: "echo catch-all".to_string(),
            is_async: false,
            timeout: 30,
            enabled: true,
        },
        HookDefinition {
            pattern: "issue.creating".to_string(),
            command: "echo specific".to_string(),
            is_async: false,
            timeout: 30,
            enabled: true,
        },
        HookDefinition {
            pattern: "*.creating".to_string(),
            command: "echo mid".to_string(),
            is_async: false,
            timeout: 30,
            enabled: true,
        },
    ];
    let result = find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Create);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].command, "echo specific"); // specificity 2
    assert_eq!(result[1].command, "echo mid"); // specificity 1
    assert_eq!(result[2].command, "echo catch-all"); // specificity 0
}

#[test]
fn test_find_matching_hooks_wildcard_matches_multiple() {
    let hooks = vec![HookDefinition {
        pattern: "*.deleted".to_string(),
        command: "echo delete".to_string(),
        is_async: false,
        timeout: 30,
        enabled: true,
    }];
    // Should match for any item type with post-delete event
    assert_eq!(
        find_matching_hooks(&hooks, Phase::Post, "issue", HookOperation::Delete).len(),
        1
    );
    assert_eq!(
        find_matching_hooks(&hooks, Phase::Post, "doc", HookOperation::Delete).len(),
        1
    );
    // pre-delete does not match "*.deleted"
    assert_eq!(
        find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Delete).len(),
        0
    );
    // post-create does not match "*.deleted"
    assert_eq!(
        find_matching_hooks(&hooks, Phase::Post, "issue", HookOperation::Create).len(),
        0
    );
}
