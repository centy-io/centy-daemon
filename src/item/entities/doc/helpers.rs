use super::error::DocError;

/// Convert a string to a URL-friendly slug
pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c == ' ' || c == '_' || c == '-' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Validate a slug
pub fn validate_slug(slug: &str) -> Result<(), DocError> {
    if slug.is_empty() {
        return Err(DocError::InvalidSlug("Slug cannot be empty".to_string()));
    }

    if !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(DocError::InvalidSlug(
            "Slug can only contain alphanumeric characters and hyphens".to_string(),
        ));
    }

    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(DocError::InvalidSlug(
            "Slug cannot start or end with a hyphen".to_string(),
        ));
    }

    Ok(())
}

/// Escape special characters in YAML strings
pub fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
