//! Migration logic for converting nested config format to flat dot-separated keys.
//!
//! Old format uses nested objects: `{ "section": { "key": false } }`
//! New format uses flat dot-separated keys: `{ "section.key": false }`
//!
//! This module can be removed once all projects have migrated to the flat format.

use serde_json::{Map, Value};
use std::collections::HashMap;

/// Config section keys whose nested objects should be flattened to dot-separated keys.
const SECTION_KEYS: &[&str] = &["workspace"];

/// Check if the raw JSON config uses the deprecated nested format for any section key.
pub fn needs_migration(raw: &Value) -> bool {
    let Some(obj) = raw.as_object() else { return false; };
    SECTION_KEYS.iter().any(|key| obj.get(*key).is_some_and(Value::is_object))
}

/// Flatten a nested config Value to use dot-separated keys for section objects.
pub fn flatten_config(value: Value) -> Value {
    let Value::Object(obj) = value else { return value; };
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
pub fn unflatten_config(value: Value) -> Value {
    let Value::Object(obj) = value else { return value; };
    let mut result = Map::new();
    let mut sections: HashMap<String, Map<String, Value>> = HashMap::new();
    for (key, val) in obj {
        let section_match = SECTION_KEYS.iter().find_map(|section_key| {
            let prefix = format!("{section_key}.");
            key.strip_prefix(&prefix).map(|sub_key| ((*section_key).to_string(), sub_key.to_string()))
        });
        if let Some((section_key, sub_key)) = section_match {
            sections.entry(section_key).or_default().insert(sub_key, val);
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
#[path = "migrate_tests.rs"]
mod tests;
