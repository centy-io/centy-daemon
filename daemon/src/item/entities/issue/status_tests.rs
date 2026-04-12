#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

#[tokio::test]
async fn test_validate_status_for_project_permissive_without_config() {
    // A path with no config file should be permissive (return Ok)
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let result = validate_status_for_project(temp_dir.path(), "issues", "any-status").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resolve_issue_status_defaults_to_open_without_config() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let status = resolve_issue_status(temp_dir.path(), None)
        .await
        .expect("Should resolve status");
    assert_eq!(status, "open");
}

#[tokio::test]
async fn test_resolve_issue_status_uses_requested_when_provided() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let status = resolve_issue_status(temp_dir.path(), Some("closed".to_string()))
        .await
        .expect("Should resolve status");
    assert_eq!(status, "closed");
}

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
    // Any status is invalid when statuses list is empty
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
