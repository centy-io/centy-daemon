use crate::config::CentyConfig;

fn is_valid_hex_color(color: &str) -> bool {
    let Ok(re) = regex::Regex::new(r"^#([0-9A-Fa-f]{3}|[0-9A-Fa-f]{6})$") else {
        return false;
    };
    re.is_match(color)
}
fn validate_colors<'a>(
    colors: impl IntoIterator<Item = (&'a String, &'a String)>,
    kind: &str,
) -> Result<(), String> {
    for (name, color) in colors {
        if !is_valid_hex_color(color) {
            return Err(format!(
                "invalid color '{color}' for {kind} '{name}': must be hex format (#RGB or #RRGGBB)"
            ));
        }
    }
    Ok(())
}

/// Validate the config and return an error message if invalid.
pub fn validate_config(config: &CentyConfig) -> Result<(), String> {
    if config.priority_levels < 1 || config.priority_levels > 10 {
        return Err("priority_levels must be between 1 and 10".to_string());
    }

    let mut field_names = std::collections::HashSet::new();
    for field in &config.custom_fields {
        if !field_names.insert(&field.name) {
            return Err(format!("duplicate custom field name: '{}'", field.name));
        }
        if field.field_type == "enum" && field.enum_values.is_empty() {
            return Err(format!(
                "custom field '{}' is of type 'enum' but has no enum_values",
                field.name
            ));
        }
    }

    validate_colors(&config.state_colors, "state")?;
    validate_colors(&config.priority_colors, "priority")?;

    for hook in &config.hooks {
        if hook.command.is_empty() {
            return Err(format!(
                "hook with pattern '{}' has an empty command",
                hook.pattern
            ));
        }
        if hook.timeout == 0 || hook.timeout > 300 {
            return Err(format!(
                "hook '{}' timeout must be between 1 and 300 seconds, got {}",
                hook.pattern, hook.timeout
            ));
        }
        if let Err(e) = crate::hooks::config::ParsedPattern::parse(&hook.pattern) {
            return Err(format!("invalid hook pattern '{}': {e}", hook.pattern));
        }
        if hook.is_async {
            let parsed = crate::hooks::config::ParsedPattern::parse(&hook.pattern)
                .map_err(|e| format!("invalid hook pattern: {e}"))?;
            if let crate::hooks::config::PatternSegment::Exact(ref phase) = parsed.phase {
                if phase == "pre" {
                    return Err(format!(
                        "hook '{}' cannot be async: pre-hooks must be synchronous",
                        hook.pattern
                    ));
                }
            }
        }
    }

    Ok(())
}
