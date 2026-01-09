use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatusError {
    #[error("Invalid status '{0}'. Allowed values: {1}")]
    InvalidStatus(String, String),
}

/// Validate that a status is in the allowed states list.
/// Returns Ok(()) if valid, or Err with allowed states for helpful error message.
pub fn validate_status(status: &str, allowed_states: &[String]) -> Result<(), StatusError> {
    if allowed_states.iter().any(|s| s == status) {
        Ok(())
    } else {
        let allowed_list = allowed_states.join(", ");
        Err(StatusError::InvalidStatus(
            status.to_string(),
            allowed_list,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_status_valid() {
        let allowed = vec!["open".to_string(), "closed".to_string()];
        assert!(validate_status("open", &allowed).is_ok());
        assert!(validate_status("closed", &allowed).is_ok());
    }

    #[test]
    fn test_validate_status_invalid_returns_error() {
        let allowed = vec!["open".to_string(), "closed".to_string()];
        let result = validate_status("unknown", &allowed);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("unknown"));
        assert!(err_msg.contains("open"));
        assert!(err_msg.contains("closed"));
    }

    #[test]
    fn test_validate_status_empty_allowed() {
        let allowed: Vec<String> = vec![];
        // Any status is invalid when allowed_states is empty
        let result = validate_status("open", &allowed);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_status_case_sensitive() {
        let allowed = vec!["open".to_string(), "closed".to_string()];
        // Should be case-sensitive
        assert!(validate_status("Open", &allowed).is_err());
        assert!(validate_status("CLOSED", &allowed).is_err());
    }

    #[test]
    fn test_error_message_format() {
        let allowed = vec![
            "open".to_string(),
            "planning".to_string(),
            "in-progress".to_string(),
            "closed".to_string(),
        ];
        let result = validate_status("wontfix", &allowed);
        let err_msg = result.unwrap_err().to_string();

        assert_eq!(
            err_msg,
            "Invalid status 'wontfix'. Allowed values: open, planning, in-progress, closed"
        );
    }
}
