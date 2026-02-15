use std::sync::LazyLock;

use crate::config::CentyConfig;

/// Static regex for validating hex colors (compiled once on first use)
#[expect(
    clippy::expect_used,
    reason = "Regex literal is compile-time constant and cannot fail"
)]
pub static HEX_COLOR_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^#([0-9A-Fa-f]{3}|[0-9A-Fa-f]{6})$")
        .expect("HEX_COLOR_REGEX is a valid regex literal")
});

/// Validate the config and return an error message if invalid.
pub fn validate_config(config: &CentyConfig) -> Result<(), String> {
    if config.allowed_states.is_empty() {
        return Err("allowed_states must not be empty".to_string());
    }

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

    for (state, color) in &config.state_colors {
        if !HEX_COLOR_REGEX.is_match(color) {
            return Err(format!(
                "invalid color '{color}' for state '{state}': must be hex format (#RGB or #RRGGBB)"
            ));
        }
    }
    for (priority, color) in &config.priority_colors {
        if !HEX_COLOR_REGEX.is_match(color) {
            return Err(format!(
                "invalid color '{color}' for priority '{priority}': must be hex format (#RGB or #RRGGBB)"
            ));
        }
    }

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
