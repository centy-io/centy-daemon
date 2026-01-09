use tracing::warn;

/// Default allowed PR statuses
pub const DEFAULT_PR_STATUSES: &[&str] = &["draft", "open", "merged", "closed"];

/// Validate that a PR status is in the allowed states list.
/// This is lenient: it logs a warning but accepts unknown states.
///
/// Returns `true` if the status is valid, `false` if not (but still accepted).
pub fn validate_pr_status(status: &str, allowed_states: &[String]) -> bool {
    let is_valid = allowed_states.iter().any(|s| s == status);
    if !is_valid {
        warn!(
            status = %status,
            allowed = ?allowed_states,
            "PR status '{}' is not in the allowed states list. Accepting anyway.",
            status
        );
    }
    is_valid
}

/// Get the default allowed PR statuses as a Vec<String>
#[must_use]
pub fn default_pr_statuses() -> Vec<String> {
    DEFAULT_PR_STATUSES
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_pr_status_valid() {
        let allowed = vec![
            "draft".to_string(),
            "open".to_string(),
            "merged".to_string(),
            "closed".to_string(),
        ];
        assert!(validate_pr_status("draft", &allowed));
        assert!(validate_pr_status("open", &allowed));
        assert!(validate_pr_status("merged", &allowed));
        assert!(validate_pr_status("closed", &allowed));
    }

    #[test]
    fn test_validate_pr_status_invalid_returns_false() {
        let allowed = vec![
            "draft".to_string(),
            "open".to_string(),
            "merged".to_string(),
            "closed".to_string(),
        ];
        // Should return false but not error
        assert!(!validate_pr_status("unknown", &allowed));
    }

    #[test]
    fn test_validate_pr_status_case_sensitive() {
        let allowed = vec!["draft".to_string(), "open".to_string()];
        // Should be case-sensitive
        assert!(!validate_pr_status("Draft", &allowed));
        assert!(!validate_pr_status("OPEN", &allowed));
    }

    #[test]
    fn test_default_pr_statuses() {
        let statuses = default_pr_statuses();
        assert_eq!(statuses.len(), 4);
        assert!(statuses.contains(&"draft".to_string()));
        assert!(statuses.contains(&"open".to_string()));
        assert!(statuses.contains(&"merged".to_string()));
        assert!(statuses.contains(&"closed".to_string()));
    }
}
