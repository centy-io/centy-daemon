/// Merge existing file content with template using `LineEnsureMerge` strategy.
/// Appends any template lines not already present in the existing content.
pub fn merge_lines_content(existing_content: &str, template_content: &str) -> String {
    let mut result = existing_content.to_string();
    for line in template_content.lines() {
        if !line.is_empty() && !existing_content.lines().any(|l| l == line) {
            if !result.ends_with('\n') && !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

/// Merge existing JSON with template using `JsonArrayMerge` strategy.
///
/// Uses JSON Merge Patch (RFC 7396) via `json-patch` for scalar fields, with
/// custom union logic for `words` and `ignorePaths` arrays.
pub fn merge_json_content(
    existing_content: &str,
    template_content: &str,
) -> Result<String, serde_json::Error> {
    let mut existing: serde_json::Value = serde_json::from_str(existing_content)?;
    let mut patch: serde_json::Value = serde_json::from_str(template_content)?;

    let (Some(existing_obj), Some(patch_obj)) =
        (existing.as_object_mut(), patch.as_object_mut())
    else {
        // If either is not an object, return the appropriate fallback.
        return if existing.is_object() {
            format_json_value(&existing)
        } else {
            format_json_value(&patch)
        };
    };

    // Pre-compute union arrays for special keys and remove them from the patch
    // so that json_patch::merge does not replace them.
    for key in &["words", "ignorePaths"] {
        let union_arr = union_string_arrays(existing_obj.get(*key), patch_obj.get(*key));
        patch_obj.remove(*key);
        if !union_arr.is_empty() {
            existing_obj.insert((*key).to_string(), serde_json::Value::Array(union_arr));
        }
    }

    // Apply JSON Merge Patch: template scalars override existing, user-added keys preserved.
    json_patch::merge(&mut existing, &patch);

    format_json_value(&existing)
}

fn union_string_arrays(
    a: Option<&serde_json::Value>,
    b: Option<&serde_json::Value>,
) -> Vec<serde_json::Value> {
    let mut set: std::collections::HashSet<String> = std::collections::HashSet::new();
    for val in [a, b].into_iter().flatten() {
        if let serde_json::Value::Array(items) = val {
            for item in items {
                if let Some(s) = item.as_str() {
                    set.insert(s.to_string());
                }
            }
        }
    }
    let mut sorted: Vec<String> = set.into_iter().collect();
    sorted.sort_by_key(|w| w.to_lowercase());
    sorted.into_iter().map(serde_json::Value::String).collect()
}

fn format_json_value(v: &serde_json::Value) -> Result<String, serde_json::Error> {
    let mut s = serde_json::to_string_pretty(v)?;
    s.push('\n');
    Ok(s)
}

/// Tests for `merge_json_content` edge cases.
#[cfg(test)]
mod merge_tests {
    use super::merge_json_content;

    #[test]
    fn test_merge_json_existing_not_object_returns_template() {
        // Existing is a JSON array (not object) → return template
        let existing = r#"["alpha", "beta"]"#;
        let template = r#"{"version": "0.2", "language": "en", "words": ["centy"]}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["version"], "0.2");
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_merge_json_existing_not_object_returns_template_for_null() {
        let existing = "null";
        let template = r#"{"version": "0.2", "words": ["centy"]}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["version"], "0.2");
    }

    #[test]
    fn test_merge_json_template_not_object_returns_existing() {
        // Template is a JSON array (not object) → return existing unchanged
        let existing = r#"{"version": "0.1", "language": "en", "words": ["alpha"]}"#;
        let template = r#"["template", "items"]"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["version"], "0.1");
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_merge_json_template_not_object_null_returns_existing() {
        let existing = r#"{"version": "0.1", "words": ["alpha"]}"#;
        let template = "null";

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["version"], "0.1");
    }

    #[test]
    fn test_merge_json_empty_words_arrays() {
        let existing = r#"{"version": "0.1", "language": "en", "words": [], "ignorePaths": []}"#;
        let template = r#"{"version": "0.2", "language": "en", "words": [], "ignorePaths": []}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["version"], "0.2");
        // Empty arrays stay empty
        let words_empty = parsed["words"].as_array().is_none_or(Vec::is_empty);
        assert!(words_empty);
    }

    #[test]
    fn test_merge_json_no_words_key_in_either() {
        let existing = r#"{"version": "0.1", "language": "en"}"#;
        let template = r#"{"version": "0.2", "language": "en"}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["version"], "0.2");
        // No words key should be added
        assert!(parsed.get("words").is_none());
    }

    #[test]
    fn test_merge_json_invalid_existing_error() {
        let result = merge_json_content("invalid json {{{", r#"{"version":"0.2"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_json_invalid_template_error() {
        let result = merge_json_content(r#"{"version":"0.1"}"#, "not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_json_output_ends_with_newline() {
        let existing = r#"{"version": "0.1", "words": ["alpha"]}"#;
        let template = r#"{"version": "0.2", "words": ["beta"]}"#;
        let result = merge_json_content(existing, template).unwrap();
        assert!(result.ends_with('\n'));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_merge_json_words_non_string_items_skipped() {
        // If words array has non-string items (numbers), they are skipped
        let existing = r#"{"version":"0.1","words":[42,"alpha"]}"#;
        let template = r#"{"version":"0.2","words":["beta"]}"#;
        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let words: Vec<&str> = parsed["words"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect();
        assert!(words.contains(&"alpha"));
        assert!(words.contains(&"beta"));
        assert!(!words.contains(&"42"));
    }
}
