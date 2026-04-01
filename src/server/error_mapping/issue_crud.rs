use super::ToStructuredError;
impl ToStructuredError for crate::item::entities::issue::IssueCrudError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::item::entities::issue::IssueCrudError;
        match self {
            IssueCrudError::IoError(_) => ("IO_ERROR", None),
            IssueCrudError::ManifestError(_) => ("MANIFEST_ERROR", None),
            IssueCrudError::JsonError(_) => ("JSON_ERROR", None),
            IssueCrudError::FrontmatterError(_) => ("FRONTMATTER_ERROR", None),
            IssueCrudError::NotInitialized => (
                "NOT_INITIALIZED",
                Some("Run 'centy init' to initialize the project"),
            ),
            IssueCrudError::IssueNotFound(_) | IssueCrudError::IssueDisplayNumberNotFound(_) => {
                ("ISSUE_NOT_FOUND", None)
            }
            IssueCrudError::IssueNotDeleted(_) => ("ISSUE_NOT_DELETED", None),
            IssueCrudError::IssueAlreadyDeleted(_) => ("ISSUE_ALREADY_DELETED", None),
            IssueCrudError::InvalidIssueFormat(_) => ("INVALID_ISSUE_FORMAT", None),
            IssueCrudError::InvalidPriority(_) => ("INVALID_PRIORITY", None),
            IssueCrudError::InvalidStatus(_) => ("INVALID_STATUS", None),
            IssueCrudError::ReconcileError(_) => ("RECONCILE_ERROR", None),
            IssueCrudError::TargetNotInitialized => (
                "TARGET_NOT_INITIALIZED",
                Some("Run 'centy init' in the target project first"),
            ),
            IssueCrudError::InvalidPriorityInTarget(_) => ("INVALID_PRIORITY_IN_TARGET", None),
            IssueCrudError::SameProject => ("SAME_PROJECT", None),
        }
    }
}
