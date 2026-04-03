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
pub fn merge_json_content(
    existing_content: &str,
    template_content: &str,
) -> Result<String, serde_json::Error> {
    let mut existing: serde_json::Value = serde_json::from_str(existing_content)?;
    let template: serde_json::Value = serde_json::from_str(template_content)?;
    let Some(existing_obj) = existing.as_object_mut() else {
        return serde_json::to_string_pretty(&template).map(|mut s| {
            s.push('\n');
            s
        });
    };
    let Some(template_obj) = template.as_object() else {
        return serde_json::to_string_pretty(&existing).map(|mut s| {
            s.push('\n');
            s
        });
    };
    for key in &["version", "language"] {
        if let Some(value) = template_obj.get(*key) {
            existing_obj.insert((*key).to_string(), value.clone());
        }
    }
    for key in &["words", "ignorePaths"] {
        let mut merged: std::collections::HashSet<String> = std::collections::HashSet::new();
        if let Some(serde_json::Value::Array(arr)) = existing_obj.get(*key) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    merged.insert(s.to_string());
                }
            }
        }
        if let Some(serde_json::Value::Array(arr)) = template_obj.get(*key) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    merged.insert(s.to_string());
                }
            }
        }
        if !merged.is_empty() {
            let mut words: Vec<String> = merged.into_iter().collect();
            words.sort_by_key(|w| w.to_lowercase());
            let json_words: Vec<serde_json::Value> =
                words.into_iter().map(serde_json::Value::String).collect();
            existing_obj.insert((*key).to_string(), serde_json::Value::Array(json_words));
        }
    }
    let mut output = serde_json::to_string_pretty(&existing)?;
    output.push('\n');
    Ok(output)
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
