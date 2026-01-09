//! Priority validation utilities.
//!
//! This module re-exports priority validation from the issue module.
//! In a future phase, this logic will be moved here.

use thiserror::Error;

/// Error type for priority validation
#[derive(Error, Debug, Clone)]
pub enum PriorityError {
    #[error("Invalid priority {priority}. Must be between 1 and {max}")]
    InvalidPriority { priority: u32, max: u32 },

    #[error("Priority levels must be at least 1")]
    InvalidLevels,
}

/// Validate a priority value against the maximum allowed levels.
///
/// # Arguments
///
/// * `priority` - The priority value to validate (1 = highest)
/// * `max_levels` - The maximum number of priority levels configured
///
/// # Returns
///
/// Ok if valid, Err with details if invalid
pub fn validate_priority(priority: u32, max_levels: u32) -> Result<(), PriorityError> {
    if max_levels < 1 {
        return Err(PriorityError::InvalidLevels);
    }
    if priority < 1 || priority > max_levels {
        return Err(PriorityError::InvalidPriority {
            priority,
            max: max_levels,
        });
    }
    Ok(())
}

/// Get the default priority for a given number of levels.
///
/// Returns the middle priority level (rounded down).
#[must_use]
pub fn default_priority(priority_levels: u32) -> u32 {
    if priority_levels <= 1 {
        1
    } else {
        priority_levels.div_ceil(2)
    }
}

/// Get a human-readable label for a priority value.
///
/// # Arguments
///
/// * `priority` - The priority value (1 = highest)
/// * `max_levels` - The maximum number of priority levels
///
/// # Returns
///
/// A string label like "high", "medium", "low", or "P1", "P2", etc.
#[must_use]
pub fn priority_label(priority: u32, max_levels: u32) -> String {
    match max_levels {
        1 => "normal".to_string(),
        2 => match priority {
            1 => "high".to_string(),
            _ => "low".to_string(),
        },
        3 => match priority {
            1 => "high".to_string(),
            2 => "medium".to_string(),
            _ => "low".to_string(),
        },
        4 => match priority {
            1 => "critical".to_string(),
            2 => "high".to_string(),
            3 => "medium".to_string(),
            _ => "low".to_string(),
        },
        _ => format!("P{priority}"),
    }
}

/// Convert a string label to a priority number.
///
/// # Arguments
///
/// * `label` - The priority label (e.g., "high", "medium", "low")
/// * `max_levels` - The maximum number of priority levels
///
/// # Returns
///
/// The numeric priority, or None if the label is not recognized
#[must_use]
pub fn label_to_priority(label: &str, max_levels: u32) -> Option<u32> {
    let label_lower = label.to_lowercase();

    // Try parsing as "P<n>" format first
    if let Some(n) = label_lower.strip_prefix('p') {
        if let Ok(num) = n.parse::<u32>() {
            if num >= 1 && num <= max_levels {
                return Some(num);
            }
        }
    }

    // Standard labels
    match label_lower.as_str() {
        "critical" | "urgent" | "blocker" => Some(1),
        "high" => {
            if max_levels >= 4 {
                Some(2)
            } else {
                Some(1)
            }
        }
        "medium" | "normal" => {
            match max_levels {
                1 => Some(1),
                2 => Some(1), // No medium in 2-level system
                3 => Some(2),
                4 => Some(3),
                _ => Some(max_levels.div_ceil(2)),
            }
        }
        "low" => Some(max_levels),
        _ => None,
    }
}

/// Migrate a string priority to a numeric value.
///
/// Used for backward compatibility with legacy string-based priorities.
#[must_use]
pub fn migrate_string_priority(priority_str: &str, max_levels: u32) -> u32 {
    label_to_priority(priority_str, max_levels).unwrap_or_else(|| default_priority(max_levels))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_priority_valid() {
        assert!(validate_priority(1, 3).is_ok());
        assert!(validate_priority(2, 3).is_ok());
        assert!(validate_priority(3, 3).is_ok());
    }

    #[test]
    fn test_validate_priority_invalid() {
        assert!(validate_priority(0, 3).is_err());
        assert!(validate_priority(4, 3).is_err());
    }

    #[test]
    fn test_default_priority() {
        assert_eq!(default_priority(1), 1);
        assert_eq!(default_priority(2), 1);
        assert_eq!(default_priority(3), 2);
        assert_eq!(default_priority(4), 2);
        assert_eq!(default_priority(5), 3);
    }

    #[test]
    fn test_priority_label() {
        assert_eq!(priority_label(1, 1), "normal");
        assert_eq!(priority_label(1, 2), "high");
        assert_eq!(priority_label(2, 2), "low");
        assert_eq!(priority_label(1, 3), "high");
        assert_eq!(priority_label(2, 3), "medium");
        assert_eq!(priority_label(3, 3), "low");
        assert_eq!(priority_label(1, 5), "P1");
    }

    #[test]
    fn test_label_to_priority() {
        assert_eq!(label_to_priority("high", 3), Some(1));
        assert_eq!(label_to_priority("medium", 3), Some(2));
        assert_eq!(label_to_priority("low", 3), Some(3));
        assert_eq!(label_to_priority("P2", 5), Some(2));
        assert_eq!(label_to_priority("invalid", 3), None);
    }

    #[test]
    fn test_migrate_string_priority() {
        assert_eq!(migrate_string_priority("high", 3), 1);
        assert_eq!(migrate_string_priority("medium", 3), 2);
        assert_eq!(migrate_string_priority("low", 3), 3);
        assert_eq!(migrate_string_priority("invalid", 3), 2); // defaults to middle
    }
}
