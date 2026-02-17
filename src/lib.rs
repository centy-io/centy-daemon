// Allow panic/unwrap/expect in tests (denied globally via Cargo.toml lints)
#![cfg_attr(
    test,
    allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic_in_result_fn,
        clippy::unwrap_in_result,
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing
    )
)]

pub mod common;
pub mod config;
pub mod hooks;
pub mod item;
pub mod link;
pub mod logging;
pub mod manifest;
pub mod metrics;
pub mod reconciliation;
pub mod registry;
pub mod server;
pub mod template;
pub mod user;
pub mod utils;
pub mod workspace;

// Re-export commonly used types
pub use config::item_type_config::{
    default_doc_config, default_issue_config, discover_item_types, read_item_type_config,
    write_item_type_config, ItemTypeRegistry,
};
pub use config::CentyConfig;
pub use hooks::{HookContext, HookDefinition, HookError};
pub use item::entities::doc::{
    create_doc, get_doc, list_docs, update_doc, CreateDocOptions, CreateDocResult, Doc, DocError,
    DocMetadata, UpdateDocOptions, UpdateDocResult,
};
pub use item::entities::issue::{
    create_issue, get_issue, list_issues, update_issue, CreateIssueOptions, CreateIssueResult,
    Issue, IssueMetadataFlat, UpdateIssueOptions, UpdateIssueResult,
};
pub use item::generic::{
    generic_create, generic_delete, generic_duplicate, generic_get, generic_list, generic_move,
    generic_restore, generic_soft_delete, generic_update, DuplicateGenericItemOptions,
};
pub use link::{
    create_link, delete_link, get_available_link_types, list_links, read_links, write_links,
    CreateLinkOptions, CreateLinkResult, CustomLinkTypeDefinition, DeleteLinkOptions,
    DeleteLinkResult, Link, LinkError, LinkTypeInfo, LinksFile, TargetType,
};
pub use manifest::{CentyManifest, ManagedFileType};
pub use reconciliation::{
    build_reconciliation_plan, execute_reconciliation, ReconciliationDecisions, ReconciliationPlan,
    ReconciliationResult,
};
pub use registry::{
    get_project_info, list_projects, track_project, untrack_project, ProjectInfo, ProjectRegistry,
    RegistryError, TrackedProject,
};
pub use server::CentyDaemonService;
pub use template::{
    DocTemplateContext, IssueTemplateContext, TemplateEngine, TemplateError, TemplateType,
};
pub use user::{
    create_user, delete_user, get_user, list_users, sync_users, update_user, CreateUserOptions,
    CreateUserResult, DeleteUserResult, GitContributor, SyncUsersFullResult, SyncUsersResult,
    UpdateUserOptions, UpdateUserResult, User, UserError,
};
pub use workspace::{
    cleanup_expired_workspaces, cleanup_workspace, create_temp_workspace, list_workspaces,
    CleanupResult, CreateWorkspaceOptions, CreateWorkspaceResult, TempWorkspaceEntry,
    WorkspaceError, WorkspaceRegistry, DEFAULT_TTL_HOURS,
};
