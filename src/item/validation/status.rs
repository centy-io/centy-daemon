//! Status validation traits and utilities.

use thiserror::Error;

/// Error type for status validation
#[derive(Error, Debug, Clone)]
#[error("Invalid status '{status}'. Allowed: {allowed:?}")]
pub struct StatusError {
    pub status: String,
    pub allowed: Vec<String>,
}

/// Validation mode for status checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationMode {
    /// Reject invalid statuses with error (used by Issue)
    #[default]
    Strict,
    /// Log warning but accept invalid statuses (used by PR)
    Lenient,
    /// No validation - accept anything (used by Doc)
    None,
}

/// Trait for items that have status validation.
///
/// Different item types can have different validation strategies:
/// - Issue: Strict validation (rejects invalid statuses)
/// - PR: Lenient validation (warns but accepts)
/// - Doc: No status concept
pub trait StatusValidator {
    /// The validation mode for this item type
    const VALIDATION_MODE: ValidationMode;

    /// Get the default allowed statuses for this item type
    fn default_statuses() -> Vec<String>;

    /// Validate a status value against allowed statuses.
    ///
    /// The behavior depends on the validation mode:
    /// - Strict: Returns Err if status is not in allowed list
    /// - Lenient: Logs warning and returns Ok even if invalid
    /// - None: Always returns Ok
    fn validate_status(status: &str, allowed: &[String]) -> Result<(), StatusError> {
        match Self::VALIDATION_MODE {
            ValidationMode::Strict => validate_strict(status, allowed),
            ValidationMode::Lenient => {
                validate_lenient(status, allowed);
                Ok(())
            }
            ValidationMode::None => Ok(()),
        }
    }
}

/// Strict validation - returns error if status is not allowed
pub fn validate_strict(status: &str, allowed: &[String]) -> Result<(), StatusError> {
    if allowed.iter().any(|s| s.eq_ignore_ascii_case(status)) {
        Ok(())
    } else {
        Err(StatusError {
            status: status.to_string(),
            allowed: allowed.to_vec(),
        })
    }
}

/// Lenient validation - logs warning but accepts any status
pub fn validate_lenient(status: &str, allowed: &[String]) {
    if !allowed.iter().any(|s| s.eq_ignore_ascii_case(status)) {
        tracing::warn!(
            status = %status,
            allowed = ?allowed,
            "Status not in allowed list, accepting anyway"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_validation_valid() {
        let allowed = vec!["open".to_string(), "closed".to_string()];
        assert!(validate_strict("open", &allowed).is_ok());
        assert!(validate_strict("OPEN", &allowed).is_ok()); // case insensitive
    }

    #[test]
    fn test_strict_validation_invalid() {
        let allowed = vec!["open".to_string(), "closed".to_string()];
        let result = validate_strict("invalid", &allowed);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status, "invalid");
        assert_eq!(err.allowed, allowed);
    }

    #[test]
    fn test_lenient_validation() {
        let allowed = vec!["open".to_string(), "closed".to_string()];
        // Should not panic even with invalid status
        validate_lenient("invalid", &allowed);
    }
}
