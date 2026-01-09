pub mod item;
pub mod common;
pub mod config;
pub mod features;
pub mod link;
pub mod llm;
pub mod logging;
pub mod manifest;
pub mod metrics;
pub mod migration;
pub mod reconciliation;
pub mod registry;
pub mod search;
pub mod server;
pub mod sync;
pub mod template;
pub mod user;
pub mod utils;
pub mod version;
pub mod workspace;

// Re-export commonly used types
pub use common::CommonMetadata;
pub use config::{CentyConfig, CustomFieldDefinition};
pub use item::entities::doc::{
    create_doc, delete_doc, get_doc, list_docs, update_doc,
    CreateDocOptions, CreateDocResult, DeleteDocResult, Doc, DocError, DocMetadata,
    UpdateDocOptions, UpdateDocResult,
};
pub use features::{
    build_compacted_refs, generate_migration_frontmatter, get_compact, get_feature_status,
    get_instruction, list_uncompacted_issues, mark_issues_compacted, save_migration, update_compact,
    CompactedIssueRef, FeatureError, FeatureStatus, MigrationFrontmatter, DEFAULT_INSTRUCTION_CONTENT,
};
pub use item::entities::issue::{
    create_issue, delete_issue, get_issue, list_issues, update_issue,
    CreateIssueOptions, CreateIssueResult, DeleteIssueResult, Issue,
    IssueMetadataFlat, UpdateIssueOptions, UpdateIssueResult,
};
pub use link::{
    create_link, delete_link, get_available_link_types, list_links, read_links, write_links,
    CreateLinkOptions, CreateLinkResult, CustomLinkTypeDefinition, DeleteLinkOptions,
    DeleteLinkResult, Link, LinkError, LinksFile, LinkTypeInfo, TargetType,
};
pub use item::entities::pr::{
    create_pr, delete_pr, get_pr, get_pr_by_display_number, list_prs, update_pr,
    CreatePrOptions, CreatePrResult, DeletePrResult, PrMetadataFlat, PullRequest,
    UpdatePrOptions, UpdatePrResult,
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
pub use template::{DocTemplateContext, IssueTemplateContext, TemplateEngine, TemplateError, TemplateType};
pub use migration::{
    create_registry, Migration, MigrationDirection, MigrationError, MigrationExecutor,
    MigrationRegistry, MigrationResult,
};
pub use version::{compare_versions, daemon_version, SemVer, VersionComparison, VersionError};
pub use llm::{
    spawn_agent, start_agent, get_effective_local_config, read_work_session, clear_work_session,
    record_work_session, AgentConfig, AgentSpawnMode, AgentType, LocalLlmConfig, LlmAction,
    LlmWorkSession, PromptBuilder,
};
pub use user::{
    create_user, delete_user, get_user, list_users, sync_users, update_user,
    CreateUserOptions, CreateUserResult, DeleteUserResult, GitContributor,
    SyncUsersFullResult, SyncUsersResult, UpdateUserOptions, UpdateUserResult,
    User, UserError,
};
pub use search::{
    advanced_search, format_query, parse_query,
    SearchError, SearchOptions, SearchResult, SearchResultIssue, SortField, SortOptions,
};
pub use workspace::{
    cleanup_expired_workspaces, cleanup_workspace, create_temp_workspace, list_workspaces,
    CleanupResult, CreateWorkspaceOptions, CreateWorkspaceResult, TempWorkspaceEntry,
    WorkspaceError, WorkspaceRegistry, DEFAULT_TTL_HOURS,
};
pub use sync::{
    CentySyncManager, ConflictInfo, ConflictResolution, MergeResult, PullResult,
    SyncError, CENTY_BRANCH,
};
