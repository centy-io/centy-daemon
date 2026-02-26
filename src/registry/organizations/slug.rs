use super::errors::OrganizationError;

/// Convert a name to a URL-friendly slug (kebab-case)
pub(crate) fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Validate a slug (must be non-empty, lowercase alphanumeric with hyphens)
pub(super) fn validate_slug(slug: &str) -> Result<(), OrganizationError> {
    if slug.is_empty() {
        return Err(OrganizationError::InvalidSlug(
            "Slug cannot be empty".to_string(),
        ));
    }

    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(OrganizationError::InvalidSlug(
            "Slug must contain only lowercase letters, numbers, and hyphens".to_string(),
        ));
    }

    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(OrganizationError::InvalidSlug(
            "Slug cannot start or end with a hyphen".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
#[path = "slug_tests.rs"]
mod tests;
