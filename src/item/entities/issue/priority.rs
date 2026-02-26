use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriorityError {
    #[error("Priority {0} is out of range. Valid values: 1 to {1}")]
    OutOfRange(u32, u32),
    #[error("Unknown priority label: {0}")]
    UnknownLabel(String),
}

/// Validate that priority is within the configured range
pub fn validate_priority(priority: u32, max_levels: u32) -> Result<(), PriorityError> {
    if priority < 1 || priority > max_levels {
        return Err(PriorityError::OutOfRange(priority, max_levels));
    }
    Ok(())
}

/// Get the default priority (middle value, or lower-middle for even counts).
/// Examples: levels=1->1, levels=2->1, levels=3->2, levels=4->2, levels=5->3
#[must_use]
pub fn default_priority(priority_levels: u32) -> u32 {
    if priority_levels == 0 { return 1; }
    priority_levels.div_ceil(2)
}

/// Get a human-readable label for a priority level.
/// 1 level: "normal", 2: high/low, 3: high/medium/low, 4: critical/high/medium/low, 5+: P1/P2/...
#[must_use]
pub fn priority_label(priority: u32, max_levels: u32) -> String {
    match max_levels {
        0 | 1 => "normal".to_string(),
        2 => match priority { 1 => "high".to_string(), _ => "low".to_string() },
        3 => match priority { 1 => "high".to_string(), 2 => "medium".to_string(), _ => "low".to_string() },
        4 => match priority {
            1 => "critical".to_string(), 2 => "high".to_string(),
            3 => "medium".to_string(), _ => "low".to_string()
        },
        _ => format!("P{priority}"),
    }
}

/// Convert a string priority label to a numeric priority. Returns None if not recognized.
#[must_use]
pub fn label_to_priority(label: &str, max_levels: u32) -> Option<u32> {
    match label.to_lowercase().as_str() {
        "critical" | "urgent" => Some(1),
        "high" => if max_levels >= 4 { Some(2) } else { Some(1) },
        "medium" | "normal" => Some(default_priority(max_levels)),
        "low" => Some(max_levels),
        _ => {
            if let Some(stripped) = label.strip_prefix('P').or_else(|| label.strip_prefix('p')) {
                stripped.parse::<u32>().ok()
            } else {
                label.parse::<u32>().ok()
            }
        }
    }
}

/// Migrate a string-based priority to the numeric system.
/// Falls back to the default priority for unrecognized values.
#[must_use]
pub fn migrate_string_priority(priority_str: &str, max_levels: u32) -> u32 {
    label_to_priority(priority_str, max_levels).unwrap_or_else(|| default_priority(max_levels))
}

#[cfg(test)]
#[path = "priority_tests_1.rs"]
mod tests_1;
#[cfg(test)]
#[path = "priority_tests_2.rs"]
mod tests_2;
