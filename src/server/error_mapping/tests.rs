use super::*;

#[test]
fn test_issue_crud_error_codes_non_empty() {
    use crate::item::entities::issue::IssueCrudError;
    let errors: Vec<IssueCrudError> = vec![
        IssueCrudError::IoError(std::io::Error::other("test")),
        IssueCrudError::NotInitialized,
        IssueCrudError::IssueNotFound("abc".into()),
        IssueCrudError::IssueDisplayNumberNotFound(1),
        IssueCrudError::IssueAlreadyDeleted("abc".into()),
        IssueCrudError::SameProject,
    ];
    for err in &errors {
        let (code, _) = err.error_code_and_tip();
        assert!(!code.is_empty(), "Code should not be empty for {err}");
    }
}

#[test]
fn test_hook_error_codes_non_empty() {
    use crate::hooks::HookError;
    let errors: Vec<HookError> = vec![
        HookError::PreHookFailed {
            pattern: "test".into(),
            exit_code: 1,
            stderr: "fail".into(),
        },
        HookError::Timeout {
            pattern: "test".into(),
            timeout_secs: 30,
        },
        HookError::ExecutionError("test".into()),
        HookError::InvalidPattern("test".into()),
    ];
    for err in &errors {
        let (code, _) = err.error_code_and_tip();
        assert!(!code.is_empty(), "Code should not be empty for {err}");
    }
}

#[test]
fn test_not_initialized_has_tip() {
    use crate::item::entities::issue::IssueCrudError;
    let err = IssueCrudError::NotInitialized;
    let (code, tip) = err.error_code_and_tip();
    assert_eq!(code, "NOT_INITIALIZED");
    assert!(tip.is_some());
    assert!(tip.unwrap().contains("centy init"));
}

#[test]
fn test_registry_error_codes() {
    use crate::registry::RegistryError;
    let err = RegistryError::HomeDirNotFound;
    let (code, _) = err.error_code_and_tip();
    assert_eq!(code, "HOME_DIR_NOT_FOUND");

    let err = RegistryError::ProjectNotFound("test".into());
    let (code, _) = err.error_code_and_tip();
    assert_eq!(code, "PROJECT_NOT_FOUND");
}

#[test]
fn test_config_error_codes() {
    use mdstore::ConfigError;
    let err = ConfigError::IoError(std::io::Error::other("test"));
    let (code, _) = err.error_code_and_tip();
    assert_eq!(code, "IO_ERROR");
}

#[test]
fn test_workspace_error_codes() {
    use crate::workspace::WorkspaceError;
    let err = WorkspaceError::GitError("not a git repository".to_string());
    let (code, _tip) = err.error_code_and_tip();
    assert_eq!(code, "WORKSPACE_GIT_ERROR");
}

#[test]
fn test_link_error_codes() {
    use crate::link::LinkError;
    let err = LinkError::SelfLink;
    let (code, tip) = err.error_code_and_tip();
    assert_eq!(code, "SELF_LINK");
    assert!(tip.is_some());
}
