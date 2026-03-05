use super::error_mapping::ToStructuredError;
// ── HookError ──────────────────────────────────────────────────────────────────
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
// ── OrgIssueError ──────────────────────────────────────────────────────────────
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
// ── OrgConfigError ─────────────────────────────────────────────────────────────
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
#[cfg(test)]
#[path = "error_mapping_tests.rs"]
mod error_mapping_tests;
