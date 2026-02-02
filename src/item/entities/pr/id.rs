//! PR ID utilities for UUID-based folder names.
//!
//! This module provides utilities for generating and validating PR IDs.
//! PR folders use UUIDs to prevent conflicts when multiple users create
//! PRs on different computers.

use uuid::Uuid;

/// Check if a string is a valid UUID
#[must_use]
pub fn is_uuid(s: &str) -> bool {
    Uuid::parse_str(s).is_ok()
}

/// Check if a folder name is a valid PR folder (UUID only, no legacy format for PRs)
#[must_use]
pub fn is_valid_pr_folder(name: &str) -> bool {
    is_uuid(name)
}

/// Check if a file name is a valid PR file (new YAML frontmatter format: {uuid}.md)
#[must_use]
pub fn is_valid_pr_file(name: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        && is_uuid(name.trim_end_matches(".md"))
}

/// Extract the PR ID from a filename (e.g., "550e8400-e29b-41d4-a716-446655440000.md" -> "550e8400-...")
#[must_use]
pub fn pr_id_from_filename(name: &str) -> Option<&str> {
    if is_valid_pr_file(name) {
        Some(name.trim_end_matches(".md"))
    } else {
        None
    }
}

/// Generate a new UUID for a PR folder
#[must_use]
pub fn generate_pr_id() -> String {
    Uuid::new_v4().to_string()
}

/// Get the short form of a PR ID (first 8 characters)
/// Useful for display purposes
#[allow(dead_code)] // Utility function for future CLI display features
#[must_use]
pub fn short_id(id: &str) -> &str {
    if id.len() >= 8 {
        &id[..8]
    } else {
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_uuid_valid() {
        assert!(is_uuid("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c"));
        assert!(is_uuid("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_is_uuid_invalid() {
        assert!(!is_uuid("not-a-uuid"));
        assert!(!is_uuid("0001"));
        assert!(!is_uuid(""));
        assert!(!is_uuid("a3f2b1c9-4d5e-6f7a-8b9c")); // incomplete
    }

    #[test]
    fn test_is_valid_pr_folder() {
        // Valid UUIDs
        assert!(is_valid_pr_folder("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c"));
        // Invalid (no legacy format for PRs)
        assert!(!is_valid_pr_folder("0001"));
        assert!(!is_valid_pr_folder("random-folder"));
        assert!(!is_valid_pr_folder(".DS_Store"));
    }

    #[test]
    fn test_is_valid_pr_file() {
        // Valid UUID.md files
        assert!(is_valid_pr_file("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c.md"));
        assert!(is_valid_pr_file("550e8400-e29b-41d4-a716-446655440000.md"));
        // Invalid
        assert!(!is_valid_pr_file("0001.md"));
        assert!(!is_valid_pr_file("random.md"));
        assert!(!is_valid_pr_file("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c")); // no .md
        assert!(!is_valid_pr_file(".md"));
    }

    #[test]
    fn test_pr_id_from_filename() {
        assert_eq!(
            pr_id_from_filename("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c.md"),
            Some("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c")
        );
        assert_eq!(pr_id_from_filename("0001.md"), None);
        assert_eq!(pr_id_from_filename("not-a-uuid"), None);
    }

    #[test]
    fn test_generate_pr_id() {
        let id = generate_pr_id();
        assert!(is_uuid(&id));
        // Ensure uniqueness
        let id2 = generate_pr_id();
        assert_ne!(id, id2);
    }

    #[test]
    fn test_short_id() {
        assert_eq!(short_id("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c"), "a3f2b1c9");
        assert_eq!(short_id("abc"), "abc");
    }
}
