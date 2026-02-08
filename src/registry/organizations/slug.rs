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
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Centy.io"), "centy-io");
        assert_eq!(slugify("My-Org"), "my-org");
        assert_eq!(slugify("test  spaces"), "test-spaces");
        assert_eq!(slugify("  leading"), "leading");
        assert_eq!(slugify("trailing  "), "trailing");
        assert_eq!(slugify("UPPERCASE"), "uppercase");
        assert_eq!(slugify("numbers123"), "numbers123");
    }

    #[test]
    fn test_validate_slug() {
        assert!(validate_slug("valid-slug").is_ok());
        assert!(validate_slug("also-valid-123").is_ok());
        assert!(validate_slug("simple").is_ok());

        assert!(validate_slug("").is_err());
        assert!(validate_slug("-start-with-hyphen").is_err());
        assert!(validate_slug("end-with-hyphen-").is_err());
        assert!(validate_slug("UPPERCASE").is_err());
        assert!(validate_slug("has spaces").is_err());
        assert!(validate_slug("has_underscore").is_err());
    }
}
