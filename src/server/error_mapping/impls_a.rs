use super::ToStructuredError;

impl ToStructuredError for crate::server::assert_service::AssertError {
    fn error_code_and_tip(&self) -> (&str, Option<&str>) {
        use crate::server::assert_service::AssertError;
        match self {
            AssertError::NotInitialized => (
                "NOT_INITIALIZED",
                Some("Run 'centy init' to initialize the project"),
            ),
        }
    }
}

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
