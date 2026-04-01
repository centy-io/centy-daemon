mod archived;
mod comment;
mod doc;
mod issue;

pub use archived::default_archived_config;
pub use comment::default_comment_config;
pub use doc::default_doc_config;
pub use issue::default_issue_config;

use super::types::ItemTypeConfig;

/// Validate an `ItemTypeConfig` for correctness.
///
/// Checks:
/// - `name` must not be empty or whitespace-only.
/// - `priorityLevels` must be > 0 when present.
/// - Every value in `statuses` must be non-empty (after trimming).
pub fn validate_item_type_config(config: &ItemTypeConfig) -> Result<(), String> {
    if config.name.trim().is_empty() {
        return Err("name must not be empty".to_string());
    }
    if let Some(levels) = config.priority_levels {
        if levels == 0 {
            return Err("priorityLevels must be greater than 0".to_string());
        }
    }
    for status in &config.statuses {
        if status.trim().is_empty() {
            return Err("status names must not be empty".to_string());
        }
    }
    Ok(())
}
