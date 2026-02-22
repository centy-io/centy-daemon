/// Trait for mapping domain errors to structured error codes and optional tips.
pub trait ToStructuredError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>);
}

// ── ItemError ──────────────────────────────────────────────────────────────────
impl ToStructuredError for crate::item::core::error::ItemError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::item::core::error::ItemError;
        match self {
            ItemError::IoError(_) => ("IO_ERROR", None),
            ItemError::NotFound(_) => ("ITEM_NOT_FOUND", None),
            ItemError::NotInitialized => (
                "NOT_INITIALIZED",
                Some("Run 'centy init' to initialize the project"),
            ),
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
            ItemError::TargetNotInitialized => (
                "TARGET_NOT_INITIALIZED",
                Some("Run 'centy init' in the target project first"),
            ),
        }
    }
}

// ── IssueError (create) ────────────────────────────────────────────────────────
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

// ── IssueCrudError ─────────────────────────────────────────────────────────────
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

// ── DocError ───────────────────────────────────────────────────────────────────
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

// ── OrganizationError ──────────────────────────────────────────────────────────
impl ToStructuredError for crate::registry::OrganizationError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::registry::OrganizationError;
        match self {
            OrganizationError::AlreadyExists(_) => ("ORG_ALREADY_EXISTS", None),
            OrganizationError::NotFound(_) => ("ORG_NOT_FOUND", None),
            OrganizationError::HasProjects(_) => ("ORG_HAS_PROJECTS", None),
            OrganizationError::InvalidSlug(_) => ("ORG_INVALID_SLUG", None),
            OrganizationError::DuplicateNameInOrganization { .. } => ("ORG_DUPLICATE_NAME", None),
            OrganizationError::RegistryError(_) => ("REGISTRY_ERROR", None),
            OrganizationError::IoError(_) => ("IO_ERROR", None),
            OrganizationError::JsonError(_) => ("JSON_ERROR", None),
        }
    }
}

// ── RegistryError ──────────────────────────────────────────────────────────────
impl ToStructuredError for crate::registry::RegistryError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::registry::RegistryError;
        match self {
            RegistryError::IoError(_) => ("IO_ERROR", None),
            RegistryError::JsonError(_) => ("JSON_ERROR", None),
            RegistryError::HomeDirNotFound => ("HOME_DIR_NOT_FOUND", None),
            RegistryError::ProjectNotFound(_) => ("PROJECT_NOT_FOUND", None),
        }
    }
}

// ── ConfigError ────────────────────────────────────────────────────────────────
impl ToStructuredError for mdstore::ConfigError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use mdstore::ConfigError;
        match self {
            ConfigError::IoError(_) => ("IO_ERROR", None),
            ConfigError::JsonError(_) => ("JSON_ERROR", None),
            ConfigError::YamlError(_) => ("YAML_ERROR", None),
        }
    }
}

// ── ManifestError ──────────────────────────────────────────────────────────────
impl ToStructuredError for crate::manifest::ManifestError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::manifest::ManifestError;
        match self {
            ManifestError::ReadError(_) => ("MANIFEST_READ_ERROR", None),
            ManifestError::ParseError(_) => ("MANIFEST_PARSE_ERROR", None),
        }
    }
}

// ── WorkspaceError ─────────────────────────────────────────────────────────────
impl ToStructuredError for crate::workspace::WorkspaceError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::workspace::WorkspaceError;
        match self {
            WorkspaceError::IoError(_) => ("IO_ERROR", None),
            WorkspaceError::GitError(_) => ("WORKSPACE_GIT_ERROR", None),
            WorkspaceError::IssueError(_) => ("ISSUE_ERROR", None),
        }
    }
}

// ── LinkError ──────────────────────────────────────────────────────────────────
impl ToStructuredError for crate::link::LinkError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::link::LinkError;
        match self {
            LinkError::IoError(_) => ("IO_ERROR", None),
            LinkError::InvalidLinkType(_) => ("INVALID_LINK_TYPE", None),
            LinkError::SourceNotFound(_, _) => ("LINK_SOURCE_NOT_FOUND", None),
            LinkError::TargetNotFound(_, _) => ("LINK_TARGET_NOT_FOUND", None),
            LinkError::LinkAlreadyExists => ("LINK_ALREADY_EXISTS", None),
            LinkError::LinkNotFound => ("LINK_NOT_FOUND", None),
            LinkError::SelfLink => ("SELF_LINK", Some("Cannot link an item to itself")),
        }
    }
}

// ── UserError ──────────────────────────────────────────────────────────────────
impl ToStructuredError for crate::user::UserError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::user::UserError;
        match self {
            UserError::IoError(_) => ("IO_ERROR", None),
            UserError::JsonError(_) => ("JSON_ERROR", None),
            UserError::ManifestError(_) => ("MANIFEST_ERROR", None),
            UserError::NotInitialized => (
                "NOT_INITIALIZED",
                Some("Run 'centy init' to initialize the project"),
            ),
            UserError::UserNotFound(_) => ("USER_NOT_FOUND", None),
            UserError::UserAlreadyExists(_) => ("USER_ALREADY_EXISTS", None),
            UserError::UserNotDeleted(_) => ("USER_NOT_DELETED", None),
            UserError::UserAlreadyDeleted(_) => ("USER_ALREADY_DELETED", None),
            UserError::InvalidUserId(_) => ("INVALID_USER_ID", None),
            UserError::NotGitRepository => (
                "NOT_GIT_REPOSITORY",
                Some("This command must be run inside a git repository"),
            ),
            UserError::GitError(_) => ("GIT_ERROR", None),
        }
    }
}

// ── AssetError ─────────────────────────────────────────────────────────────────
impl ToStructuredError for crate::item::entities::issue::AssetError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::item::entities::issue::AssetError;
        match self {
            AssetError::IoError(_) => ("IO_ERROR", None),
            AssetError::ManifestError(_) => ("MANIFEST_ERROR", None),
            AssetError::NotInitialized => (
                "NOT_INITIALIZED",
                Some("Run 'centy init' to initialize the project"),
            ),
            AssetError::IssueNotFound(_) => ("ISSUE_NOT_FOUND", None),
            AssetError::AssetNotFound(_) => ("ASSET_NOT_FOUND", None),
            AssetError::AssetAlreadyExists(_) => ("ASSET_ALREADY_EXISTS", None),
            AssetError::InvalidFilename(_) => ("INVALID_FILENAME", None),
            AssetError::UnsupportedFileType(_) => ("UNSUPPORTED_FILE_TYPE", None),
        }
    }
}

// ── PlanError (reconciliation) ─────────────────────────────────────────────────
impl ToStructuredError for crate::reconciliation::PlanError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::reconciliation::PlanError;
        match self {
            PlanError::IoError(_) => ("IO_ERROR", None),
        }
    }
}

// ── ExecuteError (reconciliation) ──────────────────────────────────────────────
impl ToStructuredError for crate::reconciliation::ExecuteError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::reconciliation::ExecuteError;
        match self {
            ExecuteError::IoError(_) => ("IO_ERROR", None),
            ExecuteError::ManifestError(_) => ("MANIFEST_ERROR", None),
            ExecuteError::PlanError(_) => ("RECONCILE_PLAN_ERROR", None),
            ExecuteError::ConfigError(_) => ("CONFIG_ERROR", None),
        }
    }
}

#[cfg(test)]
mod tests {
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
}
