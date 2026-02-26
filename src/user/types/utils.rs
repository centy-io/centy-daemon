use super::UserError;
/// Convert a name to a URL-friendly slug (kebab-case)
pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
/// Validate a user ID (must be non-empty, lowercase alphanumeric with hyphens)
pub fn validate_user_id(id: &str) -> Result<(), UserError> {
    if id.is_empty() {
        return Err(UserError::InvalidUserId("ID cannot be empty".to_string()));
    }
    if !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(UserError::InvalidUserId(
            "ID must contain only lowercase letters, numbers, and hyphens".to_string(),
        ));
    }
    if id.starts_with('-') || id.ends_with('-') {
        return Err(UserError::InvalidUserId(
            "ID cannot start or end with a hyphen".to_string(),
        ));
    }
    Ok(())
}
