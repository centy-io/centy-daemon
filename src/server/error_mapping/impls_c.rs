use super::ToStructuredError;
impl ToStructuredError for crate::item::entities::doc::DocError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::item::entities::doc::DocError;
        match self {
            DocError::IoError(_) => ("IO_ERROR", None),
            DocError::ManifestError(_) => ("MANIFEST_ERROR", None),
            DocError::JsonError(_) => ("JSON_ERROR", None),
            DocError::NotInitialized => (
                "NOT_INITIALIZED",
                Some("Run 'centy init' to initialize the project"),
            ),
            DocError::DocNotFound(_) => ("DOC_NOT_FOUND", None),
            DocError::TitleRequired => ("TITLE_REQUIRED", Some("Provide a non-empty title")),
            DocError::SlugAlreadyExists(_) => ("SLUG_ALREADY_EXISTS", None),
            DocError::InvalidSlug(_) => ("INVALID_SLUG", None),
            DocError::DocNotDeleted(_) => ("DOC_NOT_DELETED", None),
            DocError::DocAlreadyDeleted(_) => ("DOC_ALREADY_DELETED", None),
            DocError::TemplateError(_) => ("TEMPLATE_ERROR", None),
            DocError::TargetNotInitialized => (
                "TARGET_NOT_INITIALIZED",
                Some("Run 'centy init' in the target project first"),
            ),
            DocError::SameProjectMove => ("SAME_PROJECT", None),
            DocError::NoOrganization => ("NO_ORGANIZATION", None),
            DocError::RegistryError(_) => ("REGISTRY_ERROR", None),
        }
    }
}
impl ToStructuredError for crate::hooks::HookError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::hooks::HookError;
        match self {
            HookError::PreHookFailed { .. } => ("HOOK_PRE_FAILED", None),
            HookError::Timeout { .. } => ("HOOK_TIMEOUT", None),
            HookError::ExecutionError(_) => ("HOOK_EXECUTION_ERROR", None),
            HookError::InvalidPattern(_) => ("HOOK_INVALID_PATTERN", None),
            HookError::IoError(_) => ("IO_ERROR", None),
            HookError::JsonError(_) => ("JSON_ERROR", None),
        }
    }
}
impl ToStructuredError for crate::registry::OrgIssueError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::registry::OrgIssueError;
        match self {
            OrgIssueError::IoError(_) => ("IO_ERROR", None),
            OrgIssueError::JsonError(_) => ("JSON_ERROR", None),
            OrgIssueError::FrontmatterError(_) => ("FRONTMATTER_ERROR", None),
            OrgIssueError::PathError(_) => ("PATH_ERROR", None),
            OrgIssueError::OrgRegistryError(_) => ("ORG_REGISTRY_ERROR", None),
            OrgIssueError::NotFound(_) => ("ORG_ISSUE_NOT_FOUND", None),
            OrgIssueError::TitleRequired => ("TITLE_REQUIRED", Some("Provide a non-empty title")),
        }
    }
}
impl ToStructuredError for crate::registry::OrgConfigError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::registry::OrgConfigError;
        match self {
            OrgConfigError::IoError(_) => ("IO_ERROR", None),
            OrgConfigError::JsonError(_) => ("JSON_ERROR", None),
            OrgConfigError::PathError(_) => ("PATH_ERROR", None),
        }
    }
}
