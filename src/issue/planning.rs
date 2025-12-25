//! Planning state handling for issues.
//!
//! When an issue is in "planning" state, a special note is embedded in issue.md
//! instructing LLMs to brainstorm with users without implementing code changes.

/// The planning mode note that gets embedded in issue.md
pub const PLANNING_NOTE: &str = r#"> **Planning Mode**: Do not implement code changes. Brainstorm with the user, create an action plan, and update this issue before transitioning to "in-progress".

"#;

/// Planning state constant
pub const PLANNING_STATUS: &str = "planning";

/// Check if a status is the planning status
pub fn is_planning_status(status: &str) -> bool {
    status == PLANNING_STATUS
}

/// Check if issue content has the planning note
/// Handles both original and markdown-formatted versions
pub fn has_planning_note(content: &str) -> bool {
    // Check for exact match first
    if content.starts_with(PLANNING_NOTE) || content.contains(PLANNING_NOTE) {
        return true;
    }
    // Check for formatted version (markdown formatter may add spaces/newlines)
    // The distinctive marker is "> **Planning Mode**" or " > **Planning Mode**"
    content.contains("> **Planning Mode**")
}

/// Add planning note to issue content (at the top)
/// Returns the content unchanged if note already exists
pub fn add_planning_note(content: &str) -> String {
    if has_planning_note(content) {
        content.to_string()
    } else {
        format!("{PLANNING_NOTE}{content}")
    }
}

/// Remove planning note from issue content
/// Handles cases where the note may have been manually edited slightly
pub fn remove_planning_note(content: &str) -> String {
    if let Some(stripped) = content.strip_prefix(PLANNING_NOTE) {
        stripped.to_string()
    } else if content.contains(PLANNING_NOTE) {
        content.replace(PLANNING_NOTE, "")
    } else if content.contains("> **Planning Mode**") {
        // Handle edge case: partial match for manually edited notes
        // Look for the distinctive pattern and remove the entire blockquote line
        let mut result = String::new();
        let mut skip_blockquote = false;

        for line in content.lines() {
            if line.starts_with("> **Planning Mode**") {
                skip_blockquote = true;
                continue;
            }
            // Skip empty line immediately after blockquote
            if skip_blockquote && line.is_empty() {
                skip_blockquote = false;
                continue;
            }
            skip_blockquote = false;
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
        }

        result
    } else {
        // No planning note found, return content unchanged
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_planning_status() {
        assert!(is_planning_status("planning"));
        assert!(!is_planning_status("open"));
        assert!(!is_planning_status("in-progress"));
        assert!(!is_planning_status("closed"));
        assert!(!is_planning_status("Planning")); // case-sensitive
    }

    #[test]
    fn test_has_planning_note_at_start() {
        let content = format!("{PLANNING_NOTE}# Title\n\nDescription");
        assert!(has_planning_note(&content));
    }

    #[test]
    fn test_has_planning_note_false() {
        let content = "# Title\n\nDescription";
        assert!(!has_planning_note(content));
    }

    #[test]
    fn test_add_planning_note() {
        let content = "# Title\n\nDescription\n";
        let result = add_planning_note(content);
        assert!(result.starts_with(PLANNING_NOTE));
        assert!(result.contains("# Title"));
    }

    #[test]
    fn test_add_planning_note_idempotent() {
        let content = format!("{PLANNING_NOTE}# Title\n");
        let result = add_planning_note(&content);
        // Should not add duplicate note
        assert_eq!(result.matches("> **Planning Mode**").count(), 1);
    }

    #[test]
    fn test_remove_planning_note() {
        let content = format!("{PLANNING_NOTE}# Title\n\nDescription\n");
        let result = remove_planning_note(&content);
        assert!(!result.contains("Planning Mode"));
        assert!(result.starts_with("# Title"));
    }

    #[test]
    fn test_remove_planning_note_when_absent() {
        let content = "# Title\n\nDescription\n";
        let result = remove_planning_note(content);
        assert_eq!(result, content);
    }

    #[test]
    fn test_remove_manually_edited_note() {
        // User might have slightly modified the note
        let content = "> **Planning Mode**: Custom text here\n\n# Title\n";
        let result = remove_planning_note(content);
        assert!(!result.contains("Planning Mode"));
        assert!(result.contains("# Title"));
    }
}
