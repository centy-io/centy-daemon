use super::*;
use serde_json::json;

#[test]
fn test_needs_migration_no_sections() {
    let raw = json!({
        "version": "0.0.1",
        "priorityLevels": 3
    });
    assert!(!needs_migration(&raw));
}

#[test]
fn test_needs_migration_non_object() {
    assert!(!needs_migration(&json!("not an object")));
}

#[test]
fn test_flatten_preserves_non_section_objects() {
    let input = json!({
        "stateColors": { "open": "#00ff00" },
        "defaults": { "priority": "1" }
    });

    let result = flatten_config(input);
    let obj = result.as_object().unwrap();

    // These are maps, not sections — should be preserved as nested objects
    assert_eq!(obj.get("stateColors"), Some(&json!({ "open": "#00ff00" })));
    assert_eq!(obj.get("defaults"), Some(&json!({ "priority": "1" })));
}

#[test]
fn test_flatten_already_flat() {
    let input = json!({
        "version": "0.0.1",
        "priorityLevels": 3
    });

    let result = flatten_config(input.clone());
    assert_eq!(result, input);
}

#[test]
fn test_unflatten_no_section_keys() {
    let input = json!({
        "version": "0.0.1",
        "priorityLevels": 3
    });

    let result = unflatten_config(input.clone());
    assert_eq!(result, input);
}

#[test]
fn test_flatten_non_object_passthrough() {
    let input = json!("not an object");
    assert_eq!(flatten_config(input.clone()), input);
}

#[test]
fn test_unflatten_non_object_passthrough() {
    let input = json!(42);
    assert_eq!(unflatten_config(input.clone()), input);
}

#[test]
fn test_needs_migration_workspace_nested() {
    let raw = json!({
        "workspace": { "updateStatusOnOpen": true }
    });
    assert!(needs_migration(&raw));
}

#[test]
fn test_flatten_workspace_nested_to_dot() {
    let input = json!({
        "version": "0.0.1",
        "workspace": { "updateStatusOnOpen": true }
    });
    let result = flatten_config(input);
    let obj = result.as_object().unwrap();
    assert_eq!(obj.get("workspace.updateStatusOnOpen"), Some(&json!(true)));
    assert!(!obj.contains_key("workspace"));
}

#[test]
fn test_unflatten_workspace_dot_to_nested() {
    let input = json!({
        "version": "0.0.1",
        "workspace.updateStatusOnOpen": false
    });
    let result = unflatten_config(input);
    let obj = result.as_object().unwrap();
    assert_eq!(
        obj.get("workspace"),
        Some(&json!({ "updateStatusOnOpen": false }))
    );
    assert!(!obj.contains_key("workspace.updateStatusOnOpen"));
}
