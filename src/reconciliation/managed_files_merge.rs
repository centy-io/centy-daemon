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
