use super::ToStructuredError;
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
