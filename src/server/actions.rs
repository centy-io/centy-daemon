use super::proto::{ActionCategory, EntityAction};

/// Create a simple `EntityAction`.
pub fn make_action(
    id: &str,
    label: &str,
    category: i32,
    shortcut: &str,
    destructive: bool,
) -> EntityAction {
    EntityAction {
        id: id.to_string(),
        label: label.to_string(),
        category,
        enabled: true,
        disabled_reason: String::new(),
        destructive,
        keyboard_shortcut: shortcut.to_string(),
    }
}

/// Create a status action with enabled/disabled logic.
pub fn make_status_action(
    state: &str,
    entity_status: Option<&String>,
    is_pr: bool,
) -> EntityAction {
    let is_current = entity_status.map(|s| s == state).unwrap_or(false);
    let (enabled, reason) = if is_pr {
        let is_terminal = state == "merged" || state == "closed";
        let current_is_terminal = entity_status
            .map(|s| s == "merged" || s == "closed")
            .unwrap_or(false);
        if is_current {
            (false, "Already in this status".to_string())
        } else if current_is_terminal && !is_terminal {
            (false, "Cannot reopen after merge/close".to_string())
        } else {
            (true, String::new())
        }
    } else if is_current {
        (false, "Already in this status".to_string())
    } else {
        (true, String::new())
    };

    EntityAction {
        id: format!("status:{state}"),
        label: format!("Mark as {}", capitalize_first(state)),
        category: ActionCategory::Status as i32,
        enabled,
        disabled_reason: reason,
        destructive: false,
        keyboard_shortcut: String::new(),
    }
}

/// Capitalize the first letter of a string.
pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
