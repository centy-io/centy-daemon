/// Trait for mapping domain errors to structured error codes and optional tips.
pub trait ToStructuredError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>);
}

mod impls_a;
mod impls_b;
mod impls_c;
mod impls_d;

impl ToStructuredError for crate::item::entities::issue::IssueError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::item::entities::issue::IssueError;
        match self {
            IssueError::IoError(_) => ("IO_ERROR", None),
            IssueError::ManifestError(_) => ("MANIFEST_ERROR", None),
            IssueError::JsonError(_) => ("JSON_ERROR", None),
            IssueError::NotInitialized => (
                "NOT_INITIALIZED",
                Some("Run 'centy init' to initialize the project"),
            ),
            IssueError::TitleRequired => ("TITLE_REQUIRED", Some("Provide a non-empty title")),
            IssueError::InvalidPriority(_) => ("INVALID_PRIORITY", None),
            IssueError::InvalidStatus(_) => ("INVALID_STATUS", None),
            IssueError::TemplateError(_) => ("TEMPLATE_ERROR", None),
            IssueError::ReconcileError(_) => ("RECONCILE_ERROR", None),
            IssueError::NoOrganization => ("NO_ORGANIZATION", None),
            IssueError::OrgRegistryError(_) => ("ORG_REGISTRY_ERROR", None),
            IssueError::RegistryError(_) => ("REGISTRY_ERROR", None),
        }
    }
}

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
            IssueCrudError::IssueNotFound(_) => ("ISSUE_NOT_FOUND", None),
            IssueCrudError::IssueDisplayNumberNotFound(_) => ("ISSUE_NOT_FOUND", None),
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

#[cfg(test)]
mod tests;
