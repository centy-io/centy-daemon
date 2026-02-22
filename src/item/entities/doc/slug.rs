use super::error::DocError;

pub(super) fn slugify(s: &str) -> String {
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

pub(super) fn validate_slug(slug: &str) -> Result<(), DocError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Getting Started Guide"), "getting-started-guide");
        assert_eq!(slugify("API v2"), "api-v2");
        assert_eq!(slugify("  Spaces  "), "spaces");
        assert_eq!(slugify("multiple---hyphens"), "multiple-hyphens");
        assert_eq!(slugify("Under_score"), "under-score");
    }

    #[test]
    fn test_validate_slug() {
        assert!(validate_slug("hello-world").is_ok());
        assert!(validate_slug("api-v2").is_ok());
        assert!(validate_slug("").is_err());
        assert!(validate_slug("-start").is_err());
        assert!(validate_slug("end-").is_err());
        assert!(validate_slug("has space").is_err());
    }
}
