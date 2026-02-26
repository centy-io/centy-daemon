use super::*;

#[test]
fn test_hook_error_pre_hook_failed() {
    let err = HookError::PreHookFailed {
        pattern: "issue:create:*".to_string(),
        exit_code: 1,
        stderr: "validation failed".to_string(),
    };
    let display = format!("{err}");
    assert!(display.contains("Pre-hook"));
    assert!(display.contains("issue:create:*"));
    assert!(display.contains("exit code 1"));
    assert!(display.contains("validation failed"));
}

#[test]
fn test_hook_error_timeout() {
    let err = HookError::Timeout {
        pattern: "slow-hook".to_string(),
        timeout_secs: 30,
    };
    let display = format!("{err}");
    assert!(display.contains("timed out"));
    assert!(display.contains("slow-hook"));
    assert!(display.contains("30s"));
}

#[test]
fn test_hook_error_execution_error() {
    let err = HookError::ExecutionError("command not found".to_string());
    let display = format!("{err}");
    assert!(display.contains("Hook execution error"));
    assert!(display.contains("command not found"));
}

#[test]
fn test_hook_error_invalid_pattern() {
    let err = HookError::InvalidPattern("bad:pattern:[".to_string());
    let display = format!("{err}");
    assert!(display.contains("Invalid hook pattern"));
}

#[test]
fn test_hook_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err = HookError::from(io_err);
    assert!(matches!(err, HookError::IoError(_)));
}
