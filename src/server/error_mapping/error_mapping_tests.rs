use super::*;

// ── mod.rs: AssertError ───────────────────────────────────────────────────────

#[test]
fn test_assert_error_not_initialized() {
    use crate::server::assert_service::AssertError;
    let err = AssertError::NotInitialized;
    let (code, tip) = err.error_code_and_tip();
    assert_eq!(code, "NOT_INITIALIZED");
    assert!(tip.is_some());
    assert!(tip.unwrap().contains("centy init"));
}

// ── mod.rs: ItemError ────────────────────────────────────────────────────────

#[test]
fn test_item_error_all_variants() {
    use crate::item::core::error::ItemError;
    let cases: &[(&str, ItemError)] = &[
        ("IO_ERROR", ItemError::IoError(std::io::Error::other("x"))),
        ("ITEM_NOT_FOUND", ItemError::NotFound("id".into())),
        ("NOT_INITIALIZED", ItemError::NotInitialized),
        ("VALIDATION_ERROR", ItemError::ValidationError("v".into())),
        (
            "MANIFEST_ERROR",
            ItemError::ManifestError(crate::manifest::ManifestError::ReadError(
                std::io::Error::other("x"),
            )),
        ),
        (
            "JSON_ERROR",
            ItemError::JsonError(serde_json::from_str::<()>("bad").unwrap_err()),
        ),
        ("YAML_ERROR", ItemError::YamlError("y".into())),
        ("FRONTMATTER_ERROR", ItemError::FrontmatterError("f".into())),
        (
            "ITEM_TYPE_NOT_FOUND",
            ItemError::ItemTypeNotFound("t".into()),
        ),
        (
            "FEATURE_NOT_ENABLED",
            ItemError::FeatureNotEnabled("f".into()),
        ),
        ("ALREADY_DELETED", ItemError::AlreadyDeleted("a".into())),
        ("NOT_DELETED", ItemError::NotDeleted("n".into())),
        (
            "INVALID_STATUS",
            ItemError::InvalidStatus {
                status: "bad".into(),
                allowed: vec!["open".into()],
            },
        ),
        (
            "INVALID_PRIORITY",
            ItemError::InvalidPriority {
                priority: 99,
                max: 3,
            },
        ),
        ("ALREADY_EXISTS", ItemError::AlreadyExists("a".into())),
        ("IS_DELETED", ItemError::IsDeleted("d".into())),
        ("ORG_SYNC_ERROR", ItemError::OrgSyncError("e".into())),
        ("SAME_PROJECT", ItemError::SameProject),
        ("TARGET_NOT_INITIALIZED", ItemError::TargetNotInitialized),
        ("CUSTOM_ERROR", ItemError::Custom("c".into())),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(code, *expected_code, "Unexpected code for variant: {err}");
    }
}

#[test]
fn test_item_error_not_initialized_has_tip() {
    use crate::item::core::error::ItemError;
    let err = ItemError::NotInitialized;
    let (_, tip) = err.error_code_and_tip();
    assert!(tip.is_some());
    assert!(tip.unwrap().contains("centy init"));
}

#[test]
fn test_item_error_target_not_initialized_has_tip() {
    use crate::item::core::error::ItemError;
    let err = ItemError::TargetNotInitialized;
    let (_, tip) = err.error_code_and_tip();
    assert!(tip.is_some());
}

// ── mod.rs: IssueError ───────────────────────────────────────────────────────

#[test]
fn test_issue_error_all_variants() {
    use crate::item::entities::issue::priority::PriorityError;
    use crate::item::entities::issue::IssueError;
    use crate::item::entities::issue::StatusError;
    let cases: &[(&str, IssueError)] = &[
        ("IO_ERROR", IssueError::IoError(std::io::Error::other("x"))),
        (
            "MANIFEST_ERROR",
            IssueError::ManifestError(crate::manifest::ManifestError::ReadError(
                std::io::Error::other("x"),
            )),
        ),
        (
            "JSON_ERROR",
            IssueError::JsonError(serde_json::from_str::<()>("bad").unwrap_err()),
        ),
        ("NOT_INITIALIZED", IssueError::NotInitialized),
        ("TITLE_REQUIRED", IssueError::TitleRequired),
        (
            "INVALID_PRIORITY",
            IssueError::InvalidPriority(PriorityError::OutOfRange(5, 3)),
        ),
        (
            "INVALID_STATUS",
            IssueError::InvalidStatus(StatusError::InvalidStatus("bad".into(), "open".into())),
        ),
        ("NO_ORGANIZATION", IssueError::NoOrganization),
        ("REGISTRY_ERROR", IssueError::RegistryError("r".into())),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(
            code, *expected_code,
            "Unexpected code for IssueError variant: {err}"
        );
    }
}

#[test]
fn test_issue_error_not_initialized_has_tip() {
    use crate::item::entities::issue::IssueError;
    let err = IssueError::NotInitialized;
    let (_, tip) = err.error_code_and_tip();
    assert!(tip.is_some());
}

#[test]
fn test_issue_error_title_required_has_tip() {
    use crate::item::entities::issue::IssueError;
    let err = IssueError::TitleRequired;
    let (_, tip) = err.error_code_and_tip();
    assert!(tip.is_some());
}

// ── impls_a.rs: IssueCrudError ────────────────────────────────────────────────

#[test]
fn test_issue_crud_error_all_variants() {
    use crate::item::entities::issue::priority::PriorityError;
    use crate::item::entities::issue::IssueCrudError;
    use crate::item::entities::issue::StatusError;
    let cases: &[(&str, IssueCrudError)] = &[
        (
            "IO_ERROR",
            IssueCrudError::IoError(std::io::Error::other("test")),
        ),
        (
            "MANIFEST_ERROR",
            IssueCrudError::ManifestError(crate::manifest::ManifestError::ReadError(
                std::io::Error::other("x"),
            )),
        ),
        (
            "JSON_ERROR",
            IssueCrudError::JsonError(serde_json::from_str::<()>("bad").unwrap_err()),
        ),
        (
            "FRONTMATTER_ERROR",
            IssueCrudError::FrontmatterError(mdstore::FrontmatterError::InvalidFormat("x".into())),
        ),
        ("NOT_INITIALIZED", IssueCrudError::NotInitialized),
        (
            "ISSUE_NOT_FOUND",
            IssueCrudError::IssueNotFound("abc".into()),
        ),
        (
            "ISSUE_NOT_FOUND",
            IssueCrudError::IssueDisplayNumberNotFound(1),
        ),
        (
            "ISSUE_NOT_DELETED",
            IssueCrudError::IssueNotDeleted("abc".into()),
        ),
        (
            "ISSUE_ALREADY_DELETED",
            IssueCrudError::IssueAlreadyDeleted("abc".into()),
        ),
        (
            "INVALID_ISSUE_FORMAT",
            IssueCrudError::InvalidIssueFormat("f".into()),
        ),
        (
            "INVALID_PRIORITY",
            IssueCrudError::InvalidPriority(PriorityError::OutOfRange(5, 3)),
        ),
        (
            "INVALID_STATUS",
            IssueCrudError::InvalidStatus(StatusError::InvalidStatus("bad".into(), "open".into())),
        ),
        (
            "RECONCILE_ERROR",
            IssueCrudError::ReconcileError(
                crate::item::entities::issue::reconcile::ReconcileError::IoError(
                    std::io::Error::other("x"),
                ),
            ),
        ),
        (
            "TARGET_NOT_INITIALIZED",
            IssueCrudError::TargetNotInitialized,
        ),
        (
            "INVALID_PRIORITY_IN_TARGET",
            IssueCrudError::InvalidPriorityInTarget(5),
        ),
        ("SAME_PROJECT", IssueCrudError::SameProject),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(
            code, *expected_code,
            "Unexpected code for IssueCrudError: {err}"
        );
    }
}

#[test]
fn test_issue_crud_error_not_initialized_has_tip() {
    use crate::item::entities::issue::IssueCrudError;
    let err = IssueCrudError::NotInitialized;
    let (code, tip) = err.error_code_and_tip();
    assert_eq!(code, "NOT_INITIALIZED");
    assert!(tip.is_some());
    assert!(tip.unwrap().contains("centy init"));
}

#[test]
fn test_issue_crud_error_target_not_initialized_has_tip() {
    use crate::item::entities::issue::IssueCrudError;
    let err = IssueCrudError::TargetNotInitialized;
    let (_, tip) = err.error_code_and_tip();
    assert!(tip.is_some());
}

// ── impls_b.rs: OrganizationError ────────────────────────────────────────────

#[test]
fn test_organization_error_all_variants() {
    use crate::registry::{OrganizationError, RegistryError};
    let cases: &[(&str, OrganizationError)] = &[
        (
            "ORG_ALREADY_EXISTS",
            OrganizationError::AlreadyExists("acme".into()),
        ),
        ("ORG_NOT_FOUND", OrganizationError::NotFound("acme".into())),
        ("ORG_HAS_PROJECTS", OrganizationError::HasProjects(3)),
        (
            "ORG_INVALID_SLUG",
            OrganizationError::InvalidSlug("bad slug".into()),
        ),
        (
            "ORG_DUPLICATE_NAME",
            OrganizationError::DuplicateNameInOrganization {
                project_name: "proj".into(),
                org_slug: "acme".into(),
            },
        ),
        (
            "REGISTRY_ERROR",
            OrganizationError::RegistryError(RegistryError::HomeDirNotFound),
        ),
        (
            "IO_ERROR",
            OrganizationError::IoError(std::io::Error::other("x")),
        ),
        (
            "JSON_ERROR",
            OrganizationError::JsonError(serde_json::from_str::<()>("bad").unwrap_err()),
        ),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(
            code, *expected_code,
            "Unexpected code for OrganizationError: {err}"
        );
    }
}

// ── impls_b.rs: RegistryError ─────────────────────────────────────────────────

#[test]
fn test_registry_error_all_variants() {
    use crate::registry::RegistryError;
    let cases: &[(&str, RegistryError)] = &[
        (
            "IO_ERROR",
            RegistryError::IoError(std::io::Error::other("x")),
        ),
        (
            "JSON_ERROR",
            RegistryError::JsonError(serde_json::from_str::<()>("bad").unwrap_err()),
        ),
        ("HOME_DIR_NOT_FOUND", RegistryError::HomeDirNotFound),
        (
            "PROJECT_NOT_FOUND",
            RegistryError::ProjectNotFound("test".into()),
        ),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(
            code, *expected_code,
            "Unexpected code for RegistryError: {err}"
        );
    }
}

// ── impls_b.rs: ConfigError ───────────────────────────────────────────────────

#[test]
fn test_config_error_all_variants() {
    use mdstore::ConfigError;
    let cases: &[(&str, ConfigError)] = &[
        (
            "IO_ERROR",
            ConfigError::IoError(std::io::Error::other("test")),
        ),
        (
            "JSON_ERROR",
            ConfigError::JsonError(serde_json::from_str::<()>("bad").unwrap_err()),
        ),
        (
            "YAML_ERROR",
            ConfigError::YamlError(serde_yaml::from_str::<()>("a: b: c").unwrap_err()),
        ),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(
            code, *expected_code,
            "Unexpected code for ConfigError: {err}"
        );
    }
}

// ── impls_b.rs: ManifestError ─────────────────────────────────────────────────

#[test]
fn test_manifest_error_all_variants() {
    use crate::manifest::ManifestError;
    let cases: &[(&str, ManifestError)] = &[
        (
            "MANIFEST_READ_ERROR",
            ManifestError::ReadError(std::io::Error::other("x")),
        ),
        (
            "MANIFEST_PARSE_ERROR",
            ManifestError::ParseError(serde_json::from_str::<()>("bad").unwrap_err()),
        ),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(
            code, *expected_code,
            "Unexpected code for ManifestError: {err}"
        );
    }
}

// ── impls_b.rs: WorkspaceError ────────────────────────────────────────────────

#[test]
fn test_workspace_error_all_variants() {
    use crate::workspace::WorkspaceError;
    let cases: &[(&str, WorkspaceError)] = &[
        (
            "IO_ERROR",
            WorkspaceError::IoError(std::io::Error::other("x")),
        ),
        (
            "WORKSPACE_GIT_ERROR",
            WorkspaceError::GitError("not a git repo".into()),
        ),
        (
            "ISSUE_ERROR",
            WorkspaceError::IssueError(crate::item::entities::issue::IssueCrudError::SameProject),
        ),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(
            code, *expected_code,
            "Unexpected code for WorkspaceError: {err}"
        );
    }
}

// ── impls_b.rs: LinkError ─────────────────────────────────────────────────────

#[test]
fn test_link_error_all_variants() {
    use crate::link::{LinkError, TargetType};
    let cases: &[(&str, LinkError)] = &[
        ("IO_ERROR", LinkError::IoError(std::io::Error::other("x"))),
        (
            "INVALID_LINK_TYPE",
            LinkError::InvalidLinkType("bad".into()),
        ),
        (
            "LINK_SOURCE_NOT_FOUND",
            LinkError::SourceNotFound("id1".into(), TargetType::issue()),
        ),
        (
            "LINK_TARGET_NOT_FOUND",
            LinkError::TargetNotFound("id2".into(), TargetType::new("doc")),
        ),
        ("LINK_ALREADY_EXISTS", LinkError::LinkAlreadyExists),
        ("LINK_NOT_FOUND", LinkError::LinkNotFound),
        ("SELF_LINK", LinkError::SelfLink),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(code, *expected_code, "Unexpected code for LinkError: {err}");
    }
}

#[test]
fn test_link_error_self_link_has_tip() {
    use crate::link::LinkError;
    let err = LinkError::SelfLink;
    let (code, tip) = err.error_code_and_tip();
    assert_eq!(code, "SELF_LINK");
    assert!(tip.is_some());
}

// ── impls_c.rs: HookError ────────────────────────────────────────────────────

#[test]
fn test_hook_error_all_variants() {
    use crate::hooks::HookError;
    let cases: &[(&str, HookError)] = &[
        (
            "HOOK_PRE_FAILED",
            HookError::PreHookFailed {
                pattern: "test".into(),
                exit_code: 1,
                stderr: "fail".into(),
            },
        ),
        (
            "HOOK_TIMEOUT",
            HookError::Timeout {
                pattern: "test".into(),
                timeout_secs: 30,
            },
        ),
        (
            "HOOK_EXECUTION_ERROR",
            HookError::ExecutionError("test".into()),
        ),
        (
            "HOOK_INVALID_PATTERN",
            HookError::InvalidPattern("test".into()),
        ),
        ("IO_ERROR", HookError::IoError(std::io::Error::other("x"))),
        (
            "JSON_ERROR",
            HookError::JsonError(serde_json::from_str::<()>("bad").unwrap_err()),
        ),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(code, *expected_code, "Unexpected code for HookError: {err}");
    }
}

// ── impls_c.rs: OrgIssueError ────────────────────────────────────────────────

#[test]
fn test_org_issue_error_all_variants() {
    use crate::registry::OrgIssueError;
    let cases: &[(&str, OrgIssueError)] = &[
        (
            "IO_ERROR",
            OrgIssueError::IoError(std::io::Error::other("x")),
        ),
        (
            "JSON_ERROR",
            OrgIssueError::JsonError(serde_json::from_str::<()>("bad").unwrap_err()),
        ),
        (
            "FRONTMATTER_ERROR",
            OrgIssueError::FrontmatterError(mdstore::FrontmatterError::InvalidFormat("x".into())),
        ),
        ("ORG_ISSUE_NOT_FOUND", OrgIssueError::NotFound("id".into())),
        ("TITLE_REQUIRED", OrgIssueError::TitleRequired),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(
            code, *expected_code,
            "Unexpected code for OrgIssueError: {err}"
        );
    }
}

#[test]
fn test_org_issue_error_title_required_has_tip() {
    use crate::registry::OrgIssueError;
    let err = OrgIssueError::TitleRequired;
    let (_, tip) = err.error_code_and_tip();
    assert!(tip.is_some());
}

// ── impls_c.rs: OrgConfigError ───────────────────────────────────────────────

#[test]
fn test_org_config_error_all_variants() {
    use crate::registry::OrgConfigError;
    let cases: &[(&str, OrgConfigError)] = &[
        (
            "IO_ERROR",
            OrgConfigError::IoError(std::io::Error::other("x")),
        ),
        (
            "JSON_ERROR",
            OrgConfigError::JsonError(serde_json::from_str::<()>("bad").unwrap_err()),
        ),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(
            code, *expected_code,
            "Unexpected code for OrgConfigError: {err}"
        );
    }
}

// ── impls_d.rs: UserError ────────────────────────────────────────────────────

#[test]
fn test_user_error_all_variants() {
    use crate::user::UserError;
    let cases: &[(&str, UserError)] = &[
        ("IO_ERROR", UserError::IoError(std::io::Error::other("x"))),
        (
            "JSON_ERROR",
            UserError::JsonError(serde_json::from_str::<()>("bad").unwrap_err()),
        ),
        (
            "MANIFEST_ERROR",
            UserError::ManifestError(crate::manifest::ManifestError::ReadError(
                std::io::Error::other("x"),
            )),
        ),
        ("NOT_INITIALIZED", UserError::NotInitialized),
        ("USER_NOT_FOUND", UserError::UserNotFound("u".into())),
        (
            "USER_ALREADY_EXISTS",
            UserError::UserAlreadyExists("u".into()),
        ),
        ("USER_NOT_DELETED", UserError::UserNotDeleted("u".into())),
        (
            "USER_ALREADY_DELETED",
            UserError::UserAlreadyDeleted("u".into()),
        ),
        ("INVALID_USER_ID", UserError::InvalidUserId("x".into())),
        ("NOT_GIT_REPOSITORY", UserError::NotGitRepository),
        ("GIT_ERROR", UserError::GitError("e".into())),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(code, *expected_code, "Unexpected code for UserError: {err}");
    }
}

#[test]
fn test_user_error_not_initialized_has_tip() {
    use crate::user::UserError;
    let err = UserError::NotInitialized;
    let (_, tip) = err.error_code_and_tip();
    assert!(tip.is_some());
}

#[test]
fn test_user_error_not_git_repository_has_tip() {
    use crate::user::UserError;
    let err = UserError::NotGitRepository;
    let (_, tip) = err.error_code_and_tip();
    assert!(tip.is_some());
}

// ── impls_d.rs: AssetError ───────────────────────────────────────────────────

#[test]
fn test_asset_error_all_variants() {
    use crate::item::entities::issue::AssetError;
    let cases: &[(&str, AssetError)] = &[
        ("IO_ERROR", AssetError::IoError(std::io::Error::other("x"))),
        (
            "MANIFEST_ERROR",
            AssetError::ManifestError(crate::manifest::ManifestError::ReadError(
                std::io::Error::other("x"),
            )),
        ),
        ("NOT_INITIALIZED", AssetError::NotInitialized),
        ("ISSUE_NOT_FOUND", AssetError::IssueNotFound("id".into())),
        ("ASSET_NOT_FOUND", AssetError::AssetNotFound("f".into())),
        (
            "ASSET_ALREADY_EXISTS",
            AssetError::AssetAlreadyExists("f".into()),
        ),
        ("INVALID_FILENAME", AssetError::InvalidFilename("f".into())),
        (
            "UNSUPPORTED_FILE_TYPE",
            AssetError::UnsupportedFileType("exe".into()),
        ),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(
            code, *expected_code,
            "Unexpected code for AssetError: {err}"
        );
    }
}

#[test]
fn test_asset_error_not_initialized_has_tip() {
    use crate::item::entities::issue::AssetError;
    let err = AssetError::NotInitialized;
    let (_, tip) = err.error_code_and_tip();
    assert!(tip.is_some());
}

// ── impls_d.rs: PlanError ────────────────────────────────────────────────────

#[test]
fn test_plan_error_all_variants() {
    use crate::reconciliation::PlanError;
    let err = PlanError::IoError(std::io::Error::other("x"));
    let (code, _) = err.error_code_and_tip();
    assert_eq!(code, "IO_ERROR");
}

// ── impls_d.rs: ExecuteError ─────────────────────────────────────────────────

#[test]
fn test_execute_error_all_variants() {
    use crate::reconciliation::{ExecuteError, PlanError};
    let cases: &[(&str, ExecuteError)] = &[
        (
            "IO_ERROR",
            ExecuteError::IoError(std::io::Error::other("x")),
        ),
        (
            "MANIFEST_ERROR",
            ExecuteError::ManifestError(crate::manifest::ManifestError::ReadError(
                std::io::Error::other("x"),
            )),
        ),
        (
            "RECONCILE_PLAN_ERROR",
            ExecuteError::PlanError(PlanError::IoError(std::io::Error::other("x"))),
        ),
        (
            "CONFIG_ERROR",
            ExecuteError::ConfigError(mdstore::ConfigError::IoError(std::io::Error::other("x"))),
        ),
    ];
    for (expected_code, err) in cases {
        let (code, _) = err.error_code_and_tip();
        assert_eq!(
            code, *expected_code,
            "Unexpected code for ExecuteError: {err}"
        );
    }
}
