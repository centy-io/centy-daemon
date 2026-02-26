//! Issue ID utilities for UUID-based folder names.
use uuid::Uuid;

/// Check if a string is a valid UUID
#[must_use]
pub fn is_uuid(s: &str) -> bool { Uuid::parse_str(s).is_ok() }

/// Check if a string is a legacy issue number (4 digits like "0001")
#[must_use]
pub fn is_legacy_number(s: &str) -> bool { s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) }

/// Check if a folder name is a valid issue folder (UUID or legacy 4-digit)
#[must_use]
pub fn is_valid_issue_folder(name: &str) -> bool { is_uuid(name) || is_legacy_number(name) }

/// Check if a filename is a valid issue markdown file (UUID.md)
#[must_use]
pub fn is_valid_issue_file(name: &str) -> bool {
    std::path::Path::new(name).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        && is_uuid(name.trim_end_matches(".md"))
}

/// Extract the issue ID from a markdown filename (removes .md extension)
#[must_use]
pub fn issue_id_from_filename(name: &str) -> Option<&str> {
    let id = name.strip_suffix(".md")?;
    if is_uuid(id) { Some(id) } else { None }
}

/// Generate a new UUID for an issue folder
#[must_use]
pub fn generate_issue_id() -> String { Uuid::new_v4().to_string() }

/// Get the short form of an issue ID (first 8 characters)
#[must_use]
pub fn short_id(id: &str) -> &str { id.get(..8).unwrap_or(id) }

#[cfg(test)]
#[path = "id_tests.rs"]
mod tests;
