//! Migration logic for converting nested config format to flat dot-separated keys.
//!
//! The old format uses nested objects:
//! ```json
//! { "llm": { "autoCloseOnComplete": false } }
//! ```
//!
//! The new format uses flat dot-separated keys (VS Code style):
//! ```json
//! { "llm.autoCloseOnComplete": false }
//! ```
//!
//! This module can be removed once all projects have migrated to the flat format.

use serde_json::{Map, Value};
use std::collections::HashMap;

/// Config section keys whose nested objects should be flattened to dot-separated keys.
/// Add new section keys here as they are introduced.
const SECTION_KEYS: &[&str] = &["llm"];

/// Check if the raw JSON config uses the deprecated nested format for any section key.
/// Returns `true` if any section key has an object value (indicating nested format).
pub fn needs_migration(raw: &Value) -> bool {
    let Some(obj) = raw.as_object() else {
        return false;
    };
    SECTION_KEYS
        .iter()
        .any(|key| obj.get(*key).is_some_and(Value::is_object))
}

/// Flatten a nested config Value to use dot-separated keys for section objects.
///
/// Nested section objects are expanded into top-level keys with dot-separated names.
/// Non-section keys are preserved as-is.
pub fn flatten_config(value: Value) -> Value {
    let Value::Object(obj) = value else {
        return value;
    };

    let mut result = Map::new();

    for (key, val) in obj {
        if SECTION_KEYS.contains(&key.as_str()) {
            if let Value::Object(section) = val {
                for (sub_key, sub_val) in section {
                    result.insert(format!("{key}.{sub_key}"), sub_val);
                }
            } else {
                result.insert(key, val);
            }
        } else {
            result.insert(key, val);
        }
    }

    Value::Object(result)
}

/// Unflatten a flat config Value back to nested objects for serde deserialization.
///
/// Dot-separated keys belonging to known sections are grouped back into nested objects.
pub fn unflatten_config(value: Value) -> Value {
    let Value::Object(obj) = value else {
        return value;
    };

    let mut result = Map::new();
    let mut sections: HashMap<String, Map<String, Value>> = HashMap::new();

    for (key, val) in obj {
        let section_match = SECTION_KEYS.iter().find_map(|section_key| {
            let prefix = format!("{section_key}.");
            key.strip_prefix(&prefix)
                .map(|sub_key| ((*section_key).to_string(), sub_key.to_string()))
        });

        if let Some((section_key, sub_key)) = section_match {
            sections
                .entry(section_key)
                .or_default()
                .insert(sub_key, val);
        } else {
            result.insert(key, val);
        }
    }

    for (section_key, section_map) in sections {
        result.insert(section_key, Value::Object(section_map));
    }

    Value::Object(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_needs_migration_with_nested_llm() {
        let raw = json!({
            "version": "0.0.1",
            "llm": {
                "autoCloseOnComplete": false,
                "allowDirectEdits": false
            }
        });
        assert!(needs_migration(&raw));
    }

    #[test]
    fn test_needs_migration_with_flat_keys() {
        let raw = json!({
            "version": "0.0.1",
            "llm.autoCloseOnComplete": false,
            "llm.allowDirectEdits": false
        });
        assert!(!needs_migration(&raw));
    }

    #[test]
    fn test_needs_migration_with_no_llm() {
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
    fn test_flatten_nested_llm() {
        let input = json!({
            "version": "0.0.1",
            "priorityLevels": 3,
            "llm": {
                "autoCloseOnComplete": false,
                "updateStatusOnStart": false,
                "allowDirectEdits": false
            },
            "hooks": []
        });

        let result = flatten_config(input);
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("version"), Some(&json!("0.0.1")));
        assert_eq!(obj.get("priorityLevels"), Some(&json!(3)));
        assert_eq!(obj.get("llm.autoCloseOnComplete"), Some(&json!(false)));
        assert_eq!(obj.get("llm.updateStatusOnStart"), Some(&json!(false)));
        assert_eq!(obj.get("llm.allowDirectEdits"), Some(&json!(false)));
        assert!(obj.get("llm").is_none(), "nested llm key should be removed");
        assert_eq!(obj.get("hooks"), Some(&json!([])));
    }

    #[test]
    fn test_flatten_empty_section() {
        let input = json!({
            "version": "0.0.1",
            "llm": {}
        });

        let result = flatten_config(input);
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("version"), Some(&json!("0.0.1")));
        assert!(obj.get("llm").is_none());
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
            "llm.autoCloseOnComplete": false
        });

        let result = flatten_config(input.clone());
        assert_eq!(result, input);
    }

    #[test]
    fn test_unflatten_flat_keys() {
        let input = json!({
            "version": "0.0.1",
            "llm.autoCloseOnComplete": false,
            "llm.allowDirectEdits": true,
            "hooks": []
        });

        let result = unflatten_config(input);
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("version"), Some(&json!("0.0.1")));
        assert_eq!(obj.get("hooks"), Some(&json!([])));

        let llm = obj.get("llm").unwrap().as_object().unwrap();
        assert_eq!(llm.get("autoCloseOnComplete"), Some(&json!(false)));
        assert_eq!(llm.get("allowDirectEdits"), Some(&json!(true)));
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
    fn test_unflatten_already_nested() {
        // If already nested (no dot keys), unflatten should pass through
        let input = json!({
            "version": "0.0.1",
            "llm": { "autoCloseOnComplete": false }
        });

        let result = unflatten_config(input.clone());
        // "llm" is not a dot-separated key, so it passes through as-is
        assert_eq!(result, input);
    }

    #[test]
    fn test_roundtrip_flatten_unflatten() {
        let nested = json!({
            "version": "0.0.1",
            "priorityLevels": 3,
            "customFields": [],
            "defaults": {},
            "allowedStates": ["open", "closed"],
            "defaultState": "open",
            "stateColors": {},
            "priorityColors": {},
            "llm": {
                "autoCloseOnComplete": false,
                "updateStatusOnStart": false,
                "allowDirectEdits": false,
                "defaultWorkspaceMode": 0
            },
            "hooks": []
        });

        let flat = flatten_config(nested.clone());
        let restored = unflatten_config(flat);

        assert_eq!(restored, nested);
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
}
