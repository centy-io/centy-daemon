//! Tag type definitions and error types.

use crate::manifest::ManifestError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A project tag for categorizing items (issues, docs, PRs)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    /// Tag name (kebab-case, primary key)
    pub name: String,
    /// Optional hex color (e.g., "#ef4444")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// ISO timestamp when created
    pub created_at: String,
    /// Whether this is an organization-level tag
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_org_tag: bool,
    /// Organization slug (for org tags)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_slug: Option<String>,
}

/// The tags.json file structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TagsFile {
    pub tags: Vec<Tag>,
}

/// Tag-related errors
#[derive(Error, Debug)]
pub enum TagError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] ManifestError),

    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,

    #[error("Tag '{0}' not found")]
    TagNotFound(String),

    #[error("Tag '{0}' already exists")]
    TagAlreadyExists(String),

    #[error("Invalid tag name: {0}")]
    InvalidTagName(String),

    #[error("Invalid color format: {0}. Use hex format like #RRGGBB or #RGB")]
    InvalidColor(String),
}

/// Convert a tag name to a URL-friendly slug (kebab-case)
pub fn slugify_tag_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Validate a tag name (must be non-empty, lowercase alphanumeric with hyphens)
pub fn validate_tag_name(name: &str) -> Result<(), TagError> {
    if name.is_empty() {
        return Err(TagError::InvalidTagName("Name cannot be empty".to_string()));
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(TagError::InvalidTagName(
            "Name must contain only lowercase letters, numbers, and hyphens".to_string(),
        ));
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err(TagError::InvalidTagName(
            "Name cannot start or end with a hyphen".to_string(),
        ));
    }

    Ok(())
}

/// Validate a hex color string (e.g., "#RRGGBB" or "#RGB")
pub fn validate_color(color: &str) -> Result<(), TagError> {
    if !color.starts_with('#') {
        return Err(TagError::InvalidColor(color.to_string()));
    }

    let hex_part = &color[1..];

    // Must be 3 or 6 hex characters
    if hex_part.len() != 3 && hex_part.len() != 6 {
        return Err(TagError::InvalidColor(color.to_string()));
    }

    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(TagError::InvalidColor(color.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_tag_name() {
        assert_eq!(slugify_tag_name("Bug Fix"), "bug-fix");
        assert_eq!(slugify_tag_name("Feature Request"), "feature-request");
        assert_eq!(slugify_tag_name("P1-Critical"), "p1-critical");
        assert_eq!(slugify_tag_name("  spaces  "), "spaces");
        assert_eq!(slugify_tag_name("UPPERCASE"), "uppercase");
        assert_eq!(slugify_tag_name("tag123"), "tag123");
    }

    #[test]
    fn test_validate_tag_name() {
        assert!(validate_tag_name("bug").is_ok());
        assert!(validate_tag_name("feature-request").is_ok());
        assert!(validate_tag_name("p1-critical").is_ok());
        assert!(validate_tag_name("tag123").is_ok());

        assert!(validate_tag_name("").is_err());
        assert!(validate_tag_name("-starts-with-hyphen").is_err());
        assert!(validate_tag_name("ends-with-hyphen-").is_err());
        assert!(validate_tag_name("UPPERCASE").is_err());
        assert!(validate_tag_name("has spaces").is_err());
        assert!(validate_tag_name("has_underscore").is_err());
    }

    #[test]
    fn test_validate_color() {
        assert!(validate_color("#fff").is_ok());
        assert!(validate_color("#FFF").is_ok());
        assert!(validate_color("#ffffff").is_ok());
        assert!(validate_color("#FFFFFF").is_ok());
        assert!(validate_color("#ef4444").is_ok());
        assert!(validate_color("#10b981").is_ok());

        assert!(validate_color("fff").is_err());
        assert!(validate_color("#ff").is_err());
        assert!(validate_color("#ffff").is_err());
        assert!(validate_color("#fffff").is_err());
        assert!(validate_color("#gggggg").is_err());
        assert!(validate_color("").is_err());
    }

    #[test]
    fn test_tag_serialization() {
        let tag = Tag {
            name: "bug".to_string(),
            color: Some("#ef4444".to_string()),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            is_org_tag: false,
            org_slug: None,
        };

        let json = serde_json::to_string(&tag).unwrap();
        assert!(json.contains("\"name\":\"bug\""));
        assert!(json.contains("\"color\":\"#ef4444\""));
        assert!(!json.contains("\"isOrgTag\"")); // Should be skipped when false
        assert!(!json.contains("\"orgSlug\"")); // Should be skipped when None
    }

    #[test]
    fn test_tag_deserialization() {
        let json = r#"{"name":"bug","createdAt":"2025-01-01T00:00:00Z"}"#;
        let tag: Tag = serde_json::from_str(json).unwrap();
        assert_eq!(tag.name, "bug");
        assert_eq!(tag.color, None);
        assert!(!tag.is_org_tag);
        assert_eq!(tag.org_slug, None);
    }

    #[test]
    fn test_org_tag_serialization() {
        let tag = Tag {
            name: "p1-critical".to_string(),
            color: Some("#f97316".to_string()),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            is_org_tag: true,
            org_slug: Some("my-company".to_string()),
        };

        let json = serde_json::to_string(&tag).unwrap();
        assert!(json.contains("\"isOrgTag\":true"));
        assert!(json.contains("\"orgSlug\":\"my-company\""));
    }
}
