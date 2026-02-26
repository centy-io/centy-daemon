//! Planning state handling for issues.
//!
//! When an issue is in "planning" state, a special note is embedded in issue.md
//! instructing AI assistants to brainstorm with users without implementing code changes.

/// The planning mode note that gets embedded in issue.md
pub const PLANNING_NOTE: &str = r#"> **Planning Mode**: Do not implement code changes. Brainstorm with the user, create an action plan, and update this issue before transitioning to "in-progress".

"#;

/// Planning state constant
pub const PLANNING_STATUS: &str = "planning";

/// Check if a status is the planning status
pub fn is_planning_status(status: &str) -> bool { status == PLANNING_STATUS }

/// Check if issue content has the planning note (handles both original and markdown-formatted versions)
pub fn has_planning_note(content: &str) -> bool {
    if content.starts_with(PLANNING_NOTE) || content.contains(PLANNING_NOTE) { return true; }
    content.contains("> **Planning Mode**")
        || content.lines().any(|line| line.trim().starts_with("> **Planning Mode**"))
}

/// Add planning note to issue content (at the top). Returns content unchanged if note already exists.
pub fn add_planning_note(content: &str) -> String {
    if has_planning_note(content) { content.to_string() } else { format!("{PLANNING_NOTE}{content}") }
}

/// Remove planning note from issue content.
pub fn remove_planning_note(content: &str) -> String {
    if let Some(stripped) = content.strip_prefix(PLANNING_NOTE) {
        return stripped.trim_start().to_string();
    }
    if content.contains(PLANNING_NOTE) {
        return content.replace(PLANNING_NOTE, "").trim_start().to_string();
    }
    if has_planning_note(content) {
        let mut result_lines: Vec<&str> = Vec::new();
        let mut in_planning_blockquote = false;
        let mut found_planning_line = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("> **Planning Mode**") {
                in_planning_blockquote = true;
                found_planning_line = true;
                continue;
            }
            let is_blockquote_line = trimmed.starts_with("> ")
                || trimmed == ">"
                || trimmed.is_empty() && trimmed.trim() == "";
            if is_blockquote_line && !found_planning_line {
                in_planning_blockquote = true;
                continue;
            }
            if in_planning_blockquote {
                if is_blockquote_line || (trimmed.is_empty() && found_planning_line) { continue; }
                in_planning_blockquote = false;
            }
            result_lines.push(line);
        }
        result_lines.join("\n")
    } else {
        content.to_string()
    }
}

#[cfg(test)]
#[path = "planning_tests.rs"]
mod tests;
