pub mod config;
pub mod docs;
pub mod features;
pub mod issue;
pub mod llm;
pub mod manifest;
pub mod migration;
pub mod pr;
pub mod reconciliation;
pub mod registry;
pub mod server;
pub mod template;
pub mod utils;
pub mod version;

// Re-export commonly used types
pub use config::{CentyConfig, CustomFieldDefinition};
pub use docs::{
    create_doc, delete_doc, get_doc, list_docs, update_doc,
    CreateDocOptions, CreateDocResult, DeleteDocResult, Doc, DocError, DocMetadata,
    UpdateDocOptions, UpdateDocResult,
};
pub use features::{
    build_compacted_refs, generate_migration_frontmatter, get_compact, get_feature_status,
    get_instruction, list_uncompacted_issues, mark_issues_compacted, save_migration, update_compact,
    CompactedIssueRef, FeatureError, FeatureStatus, MigrationFrontmatter, DEFAULT_INSTRUCTION_CONTENT,
};
pub use issue::{
    create_issue, delete_issue, get_issue, list_issues, update_issue,
    CreateIssueOptions, CreateIssueResult, DeleteIssueResult, Issue,
    IssueMetadataFlat, UpdateIssueOptions, UpdateIssueResult,
};
pub use pr::{
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
    spawn_agent, get_effective_local_config, read_work_session, clear_work_session,
    record_work_session, AgentConfig, AgentType, LocalLlmConfig, LlmAction, LlmError,
    LlmWorkSession, PromptBuilder,
};
