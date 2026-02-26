/// Trait for mapping domain errors to structured error codes and optional tips.
pub trait ToStructuredError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>);
}

mod impls_b;
mod impls_c;
mod impls_d;

impl ToStructuredError for crate::server::assert_service::AssertError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::server::assert_service::AssertError;
        match self {
            AssertError::NotInitialized => ("NOT_INITIALIZED", Some("Run 'centy init' to initialize the project")),
        }
    }
}

impl ToStructuredError for crate::item::core::error::ItemError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::item::core::error::ItemError;
        match self {
            ItemError::IoError(_) => ("IO_ERROR", None),
            ItemError::NotFound(_) => ("ITEM_NOT_FOUND", None),
            ItemError::NotInitialized => ("NOT_INITIALIZED", Some("Run 'centy init' to initialize the project")),
            ItemError::ValidationError(_) => ("VALIDATION_ERROR", None),
            ItemError::ManifestError(_) => ("MANIFEST_ERROR", None),
            ItemError::JsonError(_) => ("JSON_ERROR", None),
            ItemError::InvalidStatus { .. } => ("INVALID_STATUS", None),
            ItemError::InvalidPriority { .. } => ("INVALID_PRIORITY", None),
            ItemError::AlreadyExists(_) => ("ALREADY_EXISTS", None),
            ItemError::IsDeleted(_) => ("IS_DELETED", None),
            ItemError::OrgSyncError(_) => ("ORG_SYNC_ERROR", None),
            ItemError::YamlError(_) => ("YAML_ERROR", None),
            ItemError::FrontmatterError(_) => ("FRONTMATTER_ERROR", None),
            ItemError::ItemTypeNotFound(_) => ("ITEM_TYPE_NOT_FOUND", None),
            ItemError::FeatureNotEnabled(_) => ("FEATURE_NOT_ENABLED", None),
            ItemError::AlreadyDeleted(_) => ("ALREADY_DELETED", None),
            ItemError::NotDeleted(_) => ("NOT_DELETED", None),
            ItemError::Custom(_) => ("CUSTOM_ERROR", None),
            ItemError::SameProject => ("SAME_PROJECT", None),
            ItemError::TargetNotInitialized => ("TARGET_NOT_INITIALIZED", Some("Run 'centy init' in the target project first")),
        }
    }
}

impl ToStructuredError for crate::item::entities::issue::IssueError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::item::entities::issue::IssueError;
        match self {
            IssueError::IoError(_) => ("IO_ERROR", None),
            IssueError::ManifestError(_) => ("MANIFEST_ERROR", None),
            IssueError::JsonError(_) => ("JSON_ERROR", None),
            IssueError::NotInitialized => ("NOT_INITIALIZED", Some("Run 'centy init' to initialize the project")),
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
            IssueCrudError::NotInitialized => ("NOT_INITIALIZED", Some("Run 'centy init' to initialize the project")),
            IssueCrudError::IssueNotFound(_) => ("ISSUE_NOT_FOUND", None),
            IssueCrudError::IssueDisplayNumberNotFound(_) => ("ISSUE_NOT_FOUND", None),
            IssueCrudError::IssueNotDeleted(_) => ("ISSUE_NOT_DELETED", None),
            IssueCrudError::IssueAlreadyDeleted(_) => ("ISSUE_ALREADY_DELETED", None),
            IssueCrudError::InvalidIssueFormat(_) => ("INVALID_ISSUE_FORMAT", None),
            IssueCrudError::InvalidPriority(_) => ("INVALID_PRIORITY", None),
            IssueCrudError::InvalidStatus(_) => ("INVALID_STATUS", None),
            IssueCrudError::ReconcileError(_) => ("RECONCILE_ERROR", None),
            IssueCrudError::TargetNotInitialized => ("TARGET_NOT_INITIALIZED", Some("Run 'centy init' in the target project first")),
            IssueCrudError::InvalidPriorityInTarget(_) => ("INVALID_PRIORITY_IN_TARGET", None),
            IssueCrudError::SameProject => ("SAME_PROJECT", None),
        }
    }
}

#[cfg(test)]
mod tests;
