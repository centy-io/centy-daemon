use super::error_mapping::ToStructuredError;
// ── UserError ──────────────────────────────────────────────────────────────────
impl ToStructuredError for crate::user::UserError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::user::UserError;
        match self {
            UserError::IoError(_) => ("IO_ERROR", None),
            UserError::JsonError(_) => ("JSON_ERROR", None),
            UserError::ManifestError(_) => ("MANIFEST_ERROR", None),
            UserError::NotInitialized => ("NOT_INITIALIZED", Some("Run 'centy init' to initialize the project")),
            UserError::UserNotFound(_) => ("USER_NOT_FOUND", None),
            UserError::UserAlreadyExists(_) => ("USER_ALREADY_EXISTS", None),
            UserError::UserNotDeleted(_) => ("USER_NOT_DELETED", None),
            UserError::UserAlreadyDeleted(_) => ("USER_ALREADY_DELETED", None),
            UserError::InvalidUserId(_) => ("INVALID_USER_ID", None),
            UserError::NotGitRepository => ("NOT_GIT_REPOSITORY", Some("This command must be run inside a git repository")),
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
            AssetError::NotInitialized => ("NOT_INITIALIZED", Some("Run 'centy init' to initialize the project")),
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
        match self { PlanError::IoError(_) => ("IO_ERROR", None) }
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
