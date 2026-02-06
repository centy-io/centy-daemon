use crate::config::{
    read_config, set_project_title as set_project_title_config, write_config, CentyConfig,
    CustomFieldDefinition as InternalCustomFieldDef, LlmConfig as InternalLlmConfig,
};
use crate::hooks::{
    run_post_hooks, run_pre_hooks, HookContext, HookDefinition as InternalHookDefinition,
    HookItemType, HookOperation, Phase,
};
use crate::item::entities::doc::{
    create_doc, delete_doc, duplicate_doc, get_doc, get_docs_by_slug, list_docs, move_doc,
    restore_doc, soft_delete_doc, update_doc, CreateDocOptions, DuplicateDocOptions,
    MoveDocOptions, UpdateDocOptions,
};
use crate::item::entities::issue::{
    // Asset imports
    add_asset,
    create_issue,
    delete_asset as delete_asset_fn,
    delete_issue,
    duplicate_issue,
    get_asset,
    get_issue,
    get_issue_by_display_number,
    get_issues_by_uuid,
    list_assets,
    list_issues,
    list_shared_assets,
    move_issue,
    priority_label,
    restore_issue,
    soft_delete_issue,
    update_issue,
    AssetInfo,
    AssetScope,
    CreateIssueOptions,
    DuplicateIssueOptions,
    MoveIssueOptions,
    UpdateIssueOptions,
};
use crate::item::entities::pr::{
    create_pr, delete_pr, get_pr, get_pr_by_display_number, get_prs_by_uuid, list_prs, restore_pr,
    soft_delete_pr, update_pr, CreatePrOptions, UpdatePrOptions,
};
use crate::link::{
    create_link, delete_link, get_available_link_types, list_links, CreateLinkOptions,
    DeleteLinkOptions, TargetType,
};
use crate::manifest::{
    read_manifest, CentyManifest as InternalManifest, ManagedFileType as InternalFileType,
};
use crate::metrics::{generate_request_id, OperationTimer};
use crate::reconciliation::{
    build_reconciliation_plan, execute_reconciliation, ReconciliationDecisions,
};
use crate::registry::{
    create_organization, delete_organization, get_organization, get_project_info,
    infer_organization_from_remote, list_organizations, list_projects, set_project_archived,
    set_project_favorite, set_project_organization, set_project_user_title, track_project_async,
    try_auto_assign_organization, untrack_project, update_organization, ListProjectsOptions,
    OrgInferenceResult, OrganizationInfo, ProjectInfo,
};
use crate::search::{advanced_search, SearchOptions, SortOptions};
use crate::user::{
    create_user as internal_create_user, delete_user as internal_delete_user,
    get_user as internal_get_user, list_users as internal_list_users,
    restore_user as internal_restore_user, soft_delete_user as internal_soft_delete_user,
    sync_users as internal_sync_users, update_user as internal_update_user, CreateUserOptions,
    UpdateUserOptions,
};
use crate::utils::{format_display_path, get_centy_path, CENTY_VERSION};
use crate::workspace::{
    cleanup_expired_workspaces as internal_cleanup_expired,
    cleanup_workspace as internal_cleanup_workspace, create_standalone_workspace,
    create_temp_workspace, get_all_editors, is_editor_available,
    list_workspaces as internal_list_workspaces, resolve_editor_id,
    terminal::{is_terminal_available, open_terminal_with_agent},
    vscode::is_vscode_available,
    CreateStandaloneWorkspaceOptions, CreateWorkspaceOptions, EditorType,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, LazyLock};
use tokio::sync::watch;
use tonic::{Request, Response, Status};
use tracing::{info, instrument};

/// Static regex for validating hex colors (compiled once on first use)
#[expect(
    clippy::expect_used,
    reason = "Regex literal is compile-time constant and cannot fail"
)]
static HEX_COLOR_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^#([0-9A-Fa-f]{3}|[0-9A-Fa-f]{6})$")
        .expect("HEX_COLOR_REGEX is a valid regex literal")
});

// Import generated protobuf types
pub mod proto {
    #![allow(clippy::pedantic)]
    #![allow(clippy::all)]
    tonic::include_proto!("centy.v1");
}

use proto::centy_daemon_server::CentyDaemon;
use proto::{
    ActionCategory, AddAssetRequest, AddAssetResponse, AdvancedSearchRequest,
    AdvancedSearchResponse, Asset, CleanupExpiredWorkspacesRequest,
    CleanupExpiredWorkspacesResponse, CloseTempWorkspaceRequest, CloseTempWorkspaceResponse,
    Config, CreateDocRequest, CreateDocResponse, CreateIssueRequest, CreateIssueResponse,
    CreateLinkRequest, CreateLinkResponse, CreateOrganizationRequest, CreateOrganizationResponse,
    CreatePrRequest, CreatePrResponse, CreateUserRequest, CreateUserResponse,
    CustomFieldDefinition, DaemonInfo, DeleteAssetRequest, DeleteAssetResponse, DeleteDocRequest,
    DeleteDocResponse, DeleteIssueRequest, DeleteIssueResponse, DeleteLinkRequest,
    DeleteLinkResponse, DeleteOrganizationRequest, DeleteOrganizationResponse, DeletePrRequest,
    DeletePrResponse, DeleteUserRequest, DeleteUserResponse, Doc, DocMetadata,
    DocWithProject as ProtoDocWithProject, DuplicateDocRequest, DuplicateDocResponse,
    DuplicateIssueRequest, DuplicateIssueResponse, EditorInfo, EditorType as ProtoEditorType,
    EntityAction, EntityType, ExecuteReconciliationRequest, FileInfo, FileType, GetAssetRequest,
    GetAssetResponse, GetAvailableLinkTypesRequest, GetAvailableLinkTypesResponse,
    GetConfigRequest, GetConfigResponse, GetDaemonInfoRequest, GetDocRequest, GetDocResponse,
    GetDocsBySlugRequest, GetDocsBySlugResponse, GetEntityActionsRequest, GetEntityActionsResponse,
    GetIssueByDisplayNumberRequest, GetIssueRequest, GetIssueResponse, GetIssuesByUuidRequest,
    GetIssuesByUuidResponse, GetManifestRequest, GetManifestResponse, GetNextIssueNumberRequest,
    GetNextIssueNumberResponse, GetNextPrNumberRequest, GetNextPrNumberResponse,
    GetOrganizationRequest, GetOrganizationResponse, GetPrByDisplayNumberRequest, GetPrRequest,
    GetPrResponse, GetProjectInfoRequest, GetProjectInfoResponse, GetPrsByUuidRequest,
    GetPrsByUuidResponse, GetReconciliationPlanRequest, GetSupportedEditorsRequest,
    GetSupportedEditorsResponse, GetUserRequest, GetUserResponse,
    GitContributor as ProtoGitContributor, HookDefinition as ProtoHookDefinition, InitRequest,
    InitResponse, IsInitializedRequest, IsInitializedResponse, Issue, IssueMetadata,
    IssueWithProject as ProtoIssueWithProject, Link as ProtoLink, LinkTargetType,
    LinkTypeDefinition, LinkTypeInfo, ListAssetsRequest, ListAssetsResponse, ListDocsRequest,
    ListDocsResponse, ListIssuesRequest, ListIssuesResponse, ListLinksRequest, ListLinksResponse,
    ListOrganizationsRequest, ListOrganizationsResponse, ListProjectsRequest, ListProjectsResponse,
    ListPrsRequest, ListPrsResponse, ListSharedAssetsRequest, ListTempWorkspacesRequest,
    ListTempWorkspacesResponse, ListUsersRequest, ListUsersResponse, LlmConfig, Manifest,
    MoveDocRequest, MoveDocResponse, MoveIssueRequest, MoveIssueResponse,
    OpenAgentInTerminalRequest, OpenAgentInTerminalResponse, OpenInTempWorkspaceRequest,
    OpenInTempWorkspaceResponse, OpenInTempWorkspaceWithEditorRequest,
    OpenStandaloneWorkspaceRequest, OpenStandaloneWorkspaceResponse,
    OpenStandaloneWorkspaceWithEditorRequest, OrgDocSyncResult,
    OrgInferenceResult as ProtoOrgInferenceResult, Organization as ProtoOrganization, PrMetadata,
    PrWithProject as ProtoPrWithProject, PullRequest, ReconciliationPlan, RegisterProjectRequest,
    RegisterProjectResponse, RestartRequest, RestartResponse, RestoreDocRequest,
    RestoreDocResponse, RestoreIssueRequest, RestoreIssueResponse, RestorePrRequest,
    RestorePrResponse, RestoreUserRequest, RestoreUserResponse,
    SearchResultIssue as ProtoSearchResultIssue, SetProjectArchivedRequest,
    SetProjectArchivedResponse, SetProjectFavoriteRequest, SetProjectFavoriteResponse,
    SetProjectOrganizationRequest, SetProjectOrganizationResponse, SetProjectTitleRequest,
    SetProjectTitleResponse, SetProjectUserTitleRequest, SetProjectUserTitleResponse,
    ShutdownRequest, ShutdownResponse, SoftDeleteDocRequest, SoftDeleteDocResponse,
    SoftDeleteIssueRequest, SoftDeleteIssueResponse, SoftDeletePrRequest, SoftDeletePrResponse,
    SoftDeleteUserRequest, SoftDeleteUserResponse, SyncUsersRequest, SyncUsersResponse,
    TempWorkspace as ProtoTempWorkspace, UntrackProjectRequest, UntrackProjectResponse,
    UpdateConfigRequest, UpdateConfigResponse, UpdateDocRequest, UpdateDocResponse,
    UpdateIssueRequest, UpdateIssueResponse, UpdateOrganizationRequest, UpdateOrganizationResponse,
    UpdatePrRequest, UpdatePrResponse, UpdateUserRequest, UpdateUserResponse, User as ProtoUser,
    WorkspaceMode,
};

/// Signal type for daemon shutdown/restart
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShutdownSignal {
    None,
    Shutdown,
    Restart,
}

// ============ Helper functions for common operations ============

/// Resolve an issue by display number or UUID.
async fn resolve_issue(
    project_path: &Path,
    issue_id: &str,
) -> Result<crate::item::entities::issue::Issue, String> {
    if let Ok(display_num) = issue_id.parse::<u32>() {
        get_issue_by_display_number(project_path, display_num)
            .await
            .map_err(|e| format!("Issue not found: {e}"))
    } else {
        get_issue(project_path, issue_id)
            .await
            .map_err(|e| format!("Issue not found: {e}"))
    }
}

/// Resolve an issue ID (display number or UUID) to a UUID string.
async fn resolve_issue_id(project_path: &Path, issue_id: &str) -> Result<String, String> {
    if let Ok(display_num) = issue_id.parse::<u32>() {
        get_issue_by_display_number(project_path, display_num)
            .await
            .map(|issue| issue.id)
            .map_err(|e| format!("Issue not found: {e}"))
    } else {
        Ok(issue_id.to_string())
    }
}

/// Resolve a PR by display number or UUID.
async fn resolve_pr(
    project_path: &Path,
    pr_id: &str,
) -> Result<crate::item::entities::pr::PullRequest, String> {
    if let Ok(display_num) = pr_id.parse::<u32>() {
        get_pr_by_display_number(project_path, display_num)
            .await
            .map_err(|e| format!("PR not found: {e}"))
    } else {
        get_pr(project_path, pr_id)
            .await
            .map_err(|e| format!("PR not found: {e}"))
    }
}

/// Resolve a PR ID (display number or UUID) to a UUID string.
async fn resolve_pr_id(project_path: &Path, pr_id: &str) -> Result<String, String> {
    if let Ok(display_num) = pr_id.parse::<u32>() {
        get_pr_by_display_number(project_path, display_num)
            .await
            .map(|pr| pr.id)
            .map_err(|e| format!("PR not found: {e}"))
    } else {
        Ok(pr_id.to_string())
    }
}

/// Create a simple EntityAction.
fn make_action(
    id: &str,
    label: &str,
    category: i32,
    shortcut: &str,
    destructive: bool,
) -> EntityAction {
    EntityAction {
        id: id.to_string(),
        label: label.to_string(),
        category,
        enabled: true,
        disabled_reason: String::new(),
        destructive,
        keyboard_shortcut: shortcut.to_string(),
    }
}

/// Create a status action with enabled/disabled logic.
fn make_status_action(state: &str, entity_status: Option<&String>, is_pr: bool) -> EntityAction {
    let is_current = entity_status.map(|s| s == state).unwrap_or(false);
    let (enabled, reason) = if is_pr {
        let is_terminal = state == "merged" || state == "closed";
        let current_is_terminal = entity_status
            .map(|s| s == "merged" || s == "closed")
            .unwrap_or(false);
        if is_current {
            (false, "Already in this status".to_string())
        } else if current_is_terminal && !is_terminal {
            (false, "Cannot reopen after merge/close".to_string())
        } else {
            (true, String::new())
        }
    } else if is_current {
        (false, "Already in this status".to_string())
    } else {
        (true, String::new())
    };

    EntityAction {
        id: format!("status:{state}"),
        label: format!("Mark as {}", capitalize_first(state)),
        category: ActionCategory::Status as i32,
        enabled,
        disabled_reason: reason,
        destructive: false,
        keyboard_shortcut: String::new(),
    }
}

/// Build issue-specific actions.
fn build_issue_actions(
    entity_status: Option<&String>,
    allowed_states: &[String],
    vscode_available: bool,
    terminal_available: bool,
    has_entity_id: bool,
) -> Vec<EntityAction> {
    let mut actions = vec![make_action(
        "create",
        "Create Issue",
        ActionCategory::Crud as i32,
        "c",
        false,
    )];

    if has_entity_id {
        actions.extend([
            make_action("delete", "Delete", ActionCategory::Crud as i32, "d", true),
            make_action(
                "duplicate",
                "Duplicate",
                ActionCategory::Crud as i32,
                "D",
                false,
            ),
            make_action(
                "move",
                "Move to Project",
                ActionCategory::Crud as i32,
                "m",
                false,
            ),
            make_action("mode:plan", "Plan", ActionCategory::Mode as i32, "p", false),
            make_action(
                "mode:implement",
                "Implement",
                ActionCategory::Mode as i32,
                "i",
                false,
            ),
            make_action(
                "mode:deepdive",
                "Deep Dive",
                ActionCategory::Mode as i32,
                "D",
                false,
            ),
        ]);
        for state in allowed_states {
            actions.push(make_status_action(state, entity_status, false));
        }
        actions.push(EntityAction {
            id: "open_in_vscode".to_string(),
            label: "Open in VSCode".to_string(),
            category: ActionCategory::External as i32,
            enabled: vscode_available,
            disabled_reason: if vscode_available {
                String::new()
            } else {
                "VSCode not available".to_string()
            },
            destructive: false,
            keyboard_shortcut: "o".to_string(),
        });
        actions.push(EntityAction {
            id: "open_in_terminal".to_string(),
            label: "Open in Terminal".to_string(),
            category: ActionCategory::External as i32,
            enabled: terminal_available,
            disabled_reason: if terminal_available {
                String::new()
            } else {
                "Terminal not available".to_string()
            },
            destructive: false,
            keyboard_shortcut: "t".to_string(),
        });
    }
    actions
}

/// Build PR-specific actions.
fn build_pr_actions(entity_status: Option<&String>, has_entity_id: bool) -> Vec<EntityAction> {
    let mut actions = vec![make_action(
        "create",
        "Create PR",
        ActionCategory::Crud as i32,
        "c",
        false,
    )];
    if has_entity_id {
        actions.push(make_action(
            "delete",
            "Delete",
            ActionCategory::Crud as i32,
            "d",
            true,
        ));
        for state in ["draft", "open", "merged", "closed"] {
            actions.push(make_status_action(state, entity_status, true));
        }
    }
    actions
}

/// Build doc-specific actions.
fn build_doc_actions(has_entity_id: bool) -> Vec<EntityAction> {
    let mut actions = vec![make_action(
        "create",
        "Create Doc",
        ActionCategory::Crud as i32,
        "c",
        false,
    )];
    if has_entity_id {
        actions.extend([
            make_action("delete", "Delete", ActionCategory::Crud as i32, "d", true),
            make_action(
                "duplicate",
                "Duplicate",
                ActionCategory::Crud as i32,
                "D",
                false,
            ),
            make_action(
                "move",
                "Move to Project",
                ActionCategory::Crud as i32,
                "m",
                false,
            ),
        ]);
    }
    actions
}

pub struct CentyDaemonService {
    shutdown_tx: Arc<watch::Sender<ShutdownSignal>>,
    exe_path: Option<PathBuf>,
}

impl CentyDaemonService {
    #[must_use]
    pub fn new(shutdown_tx: Arc<watch::Sender<ShutdownSignal>>, exe_path: Option<PathBuf>) -> Self {
        // Spawn background task to infer organizations for ungrouped projects on startup
        tokio::spawn(async {
            startup_org_inference().await;
        });

        Self {
            shutdown_tx,
            exe_path,
        }
    }
}

/// Background task to infer organizations for ungrouped projects on daemon startup
async fn startup_org_inference() {
    use tokio::time::{sleep, Duration};

    // Small delay to let the daemon fully initialize
    sleep(Duration::from_millis(100)).await;

    // List all ungrouped projects that exist on disk
    let projects = match list_projects(ListProjectsOptions {
        include_stale: false,
        include_uninitialized: true,
        include_archived: false,
        ungrouped_only: true,
        ..Default::default()
    })
    .await
    {
        Ok(p) => p,
        Err(e) => {
            info!("Startup org inference: failed to list projects: {e}");
            return;
        }
    };

    if projects.is_empty() {
        return;
    }

    info!(
        "Startup org inference: scanning {} ungrouped projects",
        projects.len()
    );

    let mut inferred_count = 0;
    for project in projects {
        // Small delay between projects to avoid overloading
        sleep(Duration::from_millis(50)).await;

        if let Some(result) = try_auto_assign_organization(&project.path, None).await {
            if result.inferred_org_slug.is_some() && !result.has_mismatch {
                inferred_count += 1;
            }
        }
    }

    if inferred_count > 0 {
        info!(
            "Startup org inference: assigned organizations to {} projects",
            inferred_count
        );
    }
}

#[tonic::async_trait]
impl CentyDaemon for CentyDaemonService {
    #[instrument(
        name = "grpc.init",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn init(&self, request: Request<InitRequest>) -> Result<Response<InitResponse>, Status> {
        let _timer = OperationTimer::new("init");
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let decisions = req
            .decisions
            .map(|d| ReconciliationDecisions {
                restore: d.restore.into_iter().collect(),
                reset: d.reset.into_iter().collect(),
            })
            .unwrap_or_default();

        match execute_reconciliation(project_path, decisions, req.force).await {
            Ok(result) => {
                // Infer organization from git remote
                let existing_org = get_project_info(&req.project_path)
                    .await
                    .ok()
                    .flatten()
                    .and_then(|info| info.organization_slug);
                let inference =
                    infer_organization_from_remote(project_path, existing_org.as_deref()).await;

                // Auto-assign if no existing org and inference succeeded without mismatch
                if existing_org.is_none() && !inference.has_mismatch {
                    if let Some(ref slug) = inference.inferred_org_slug {
                        let _ = set_project_organization(&req.project_path, Some(slug)).await;
                    }
                }

                Ok(Response::new(InitResponse {
                    success: true,
                    error: String::new(),
                    created: result.created,
                    restored: result.restored,
                    reset: result.reset,
                    skipped: result.skipped,
                    manifest: Some(manifest_to_proto(&result.manifest)),
                    org_inference: Some(org_inference_to_proto(&inference)),
                }))
            }
            Err(e) => Ok(Response::new(InitResponse {
                success: false,
                error: e.to_string(),
                created: vec![],
                restored: vec![],
                reset: vec![],
                skipped: vec![],
                manifest: None,
                org_inference: None,
            })),
        }
    }

    async fn get_reconciliation_plan(
        &self,
        request: Request<GetReconciliationPlanRequest>,
    ) -> Result<Response<ReconciliationPlan>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match build_reconciliation_plan(project_path).await {
            Ok(plan) => {
                let needs_decisions = plan.needs_decisions();
                Ok(Response::new(ReconciliationPlan {
                    to_create: plan.to_create.into_iter().map(file_info_to_proto).collect(),
                    to_restore: plan
                        .to_restore
                        .into_iter()
                        .map(file_info_to_proto)
                        .collect(),
                    to_reset: plan.to_reset.into_iter().map(file_info_to_proto).collect(),
                    up_to_date: plan
                        .up_to_date
                        .into_iter()
                        .map(file_info_to_proto)
                        .collect(),
                    user_files: plan
                        .user_files
                        .into_iter()
                        .map(file_info_to_proto)
                        .collect(),
                    needs_decisions,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn execute_reconciliation(
        &self,
        request: Request<ExecuteReconciliationRequest>,
    ) -> Result<Response<InitResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let decisions = req
            .decisions
            .map(|d| ReconciliationDecisions {
                restore: d.restore.into_iter().collect(),
                reset: d.reset.into_iter().collect(),
            })
            .unwrap_or_default();

        match execute_reconciliation(project_path, decisions, false).await {
            Ok(result) => {
                // Infer organization from git remote
                let existing_org = get_project_info(&req.project_path)
                    .await
                    .ok()
                    .flatten()
                    .and_then(|info| info.organization_slug);
                let inference =
                    infer_organization_from_remote(project_path, existing_org.as_deref()).await;

                // Auto-assign if no existing org and inference succeeded without mismatch
                if existing_org.is_none() && !inference.has_mismatch {
                    if let Some(ref slug) = inference.inferred_org_slug {
                        let _ = set_project_organization(&req.project_path, Some(slug)).await;
                    }
                }

                Ok(Response::new(InitResponse {
                    success: true,
                    error: String::new(),
                    created: result.created,
                    restored: result.restored,
                    reset: result.reset,
                    skipped: result.skipped,
                    manifest: Some(manifest_to_proto(&result.manifest)),
                    org_inference: Some(org_inference_to_proto(&inference)),
                }))
            }
            Err(e) => Ok(Response::new(InitResponse {
                success: false,
                error: e.to_string(),
                created: vec![],
                restored: vec![],
                reset: vec![],
                skipped: vec![],
                manifest: None,
                org_inference: None,
            })),
        }
    }

    #[instrument(
        name = "grpc.create_issue",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn create_issue(
        &self,
        request: Request<CreateIssueRequest>,
    ) -> Result<Response<CreateIssueResponse>, Status> {
        let _timer = OperationTimer::new("create_issue");
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_request_data = serde_json::json!({
            "title": &req.title,
            "description": &req.description,
            "priority": req.priority,
            "status": &req.status,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Issue,
            HookOperation::Create,
            &hook_project_path,
            None,
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(CreateIssueResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        // Convert int32 priority: 0 means use default, otherwise use the value
        let options = CreateIssueOptions {
            title: req.title,
            description: req.description,
            priority: if req.priority == 0 {
                None
            } else {
                Some(req.priority as u32)
            },
            status: if req.status.is_empty() {
                None
            } else {
                Some(req.status)
            },
            custom_fields: req.custom_fields,
            template: if req.template.is_empty() {
                None
            } else {
                Some(req.template)
            },
            draft: Some(req.draft),
            is_org_issue: req.is_org_issue,
        };

        match create_issue(project_path, options).await {
            #[allow(deprecated)]
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Issue,
                    HookOperation::Create,
                    &hook_project_path,
                    Some(&result.id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                // Convert sync results to proto (reusing OrgDocSyncResult since structure is identical)
                let sync_results: Vec<OrgDocSyncResult> = result
                    .sync_results
                    .into_iter()
                    .map(|r| OrgDocSyncResult {
                        project_path: r.project_path,
                        success: r.success,
                        error: r.error.unwrap_or_default(),
                    })
                    .collect();

                Ok(Response::new(CreateIssueResponse {
                    success: true,
                    error: String::new(),
                    id: result.id.clone(),
                    display_number: result.display_number,
                    issue_number: result.issue_number, // Legacy
                    created_files: result.created_files,
                    manifest: Some(manifest_to_proto(&result.manifest)),
                    org_display_number: result.org_display_number.unwrap_or(0),
                    sync_results,
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Issue,
                    HookOperation::Create,
                    &hook_project_path,
                    None,
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(CreateIssueResponse {
                    success: false,
                    error: e.to_string(),
                    id: String::new(),
                    display_number: 0,
                    issue_number: String::new(),
                    created_files: vec![],
                    manifest: None,
                    org_display_number: 0,
                    sync_results: vec![],
                }))
            }
        }
    }

    #[instrument(
        name = "grpc.get_issue",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn get_issue(
        &self,
        request: Request<GetIssueRequest>,
    ) -> Result<Response<GetIssueResponse>, Status> {
        let _timer = OperationTimer::new("get_issue");
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        match resolve_issue(project_path, &req.issue_id).await {
            Ok(issue) => Ok(Response::new(GetIssueResponse {
                success: true,
                error: String::new(),
                issue: Some(issue_to_proto(&issue, priority_levels)),
            })),
            Err(e) => Ok(Response::new(GetIssueResponse {
                success: false,
                error: e,
                issue: None,
            })),
        }
    }

    async fn get_issue_by_display_number(
        &self,
        request: Request<GetIssueByDisplayNumberRequest>,
    ) -> Result<Response<GetIssueResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        match get_issue_by_display_number(project_path, req.display_number).await {
            Ok(issue) => Ok(Response::new(GetIssueResponse {
                success: true,
                error: String::new(),
                issue: Some(issue_to_proto(&issue, priority_levels)),
            })),
            Err(e) => Ok(Response::new(GetIssueResponse {
                success: false,
                error: e.to_string(),
                issue: None,
            })),
        }
    }

    async fn get_issues_by_uuid(
        &self,
        request: Request<GetIssuesByUuidRequest>,
    ) -> Result<Response<GetIssuesByUuidResponse>, Status> {
        let req = request.into_inner();

        // Get all initialized projects from registry
        let projects = match list_projects(ListProjectsOptions::default()).await {
            Ok(p) => p,
            Err(e) => return Err(Status::internal(format!("Failed to list projects: {e}"))),
        };

        match get_issues_by_uuid(&req.uuid, &projects).await {
            Ok(result) => {
                let issues_with_projects: Vec<ProtoIssueWithProject> = result
                    .issues
                    .into_iter()
                    .map(|iwp| {
                        // Use default priority_levels of 3 for global search
                        let priority_levels = 3;

                        ProtoIssueWithProject {
                            issue: Some(issue_to_proto(&iwp.issue, priority_levels)),
                            display_path: format_display_path(&iwp.project_path),
                            project_path: iwp.project_path,
                            project_name: iwp.project_name,
                        }
                    })
                    .collect();

                let total_count = issues_with_projects.len() as i32;

                Ok(Response::new(GetIssuesByUuidResponse {
                    issues: issues_with_projects,
                    total_count,
                    errors: result.errors,
                }))
            }
            Err(e) => Err(Status::invalid_argument(e.to_string())),
        }
    }

    #[instrument(
        name = "grpc.list_issues",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn list_issues(
        &self,
        request: Request<ListIssuesRequest>,
    ) -> Result<Response<ListIssuesResponse>, Status> {
        let _timer = OperationTimer::new("list_issues");
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        let status_filter = if req.status.is_empty() {
            None
        } else {
            Some(req.status.as_str())
        };
        // Convert int32 priority filter: 0 means no filter
        let priority_filter = if req.priority == 0 {
            None
        } else {
            Some(req.priority as u32)
        };
        // Draft filter is optional bool
        let draft_filter = req.draft;

        match list_issues(
            project_path,
            status_filter,
            priority_filter,
            draft_filter,
            false,
        )
        .await
        {
            Ok(issues) => {
                let total_count = issues.len() as i32;
                Ok(Response::new(ListIssuesResponse {
                    issues: issues
                        .into_iter()
                        .map(|i| issue_to_proto(&i, priority_levels))
                        .collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_issue(
        &self,
        request: Request<UpdateIssueRequest>,
    ) -> Result<Response<UpdateIssueResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.issue_id.clone();
        let hook_request_data = serde_json::json!({
            "issue_id": &req.issue_id,
            "title": &req.title,
            "description": &req.description,
            "priority": req.priority,
            "status": &req.status,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Issue,
            HookOperation::Update,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(UpdateIssueResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        // Convert int32 priority: 0 means don't update, otherwise use the value
        let options = UpdateIssueOptions {
            title: if req.title.is_empty() {
                None
            } else {
                Some(req.title)
            },
            description: if req.description.is_empty() {
                None
            } else {
                Some(req.description)
            },
            status: if req.status.is_empty() {
                None
            } else {
                Some(req.status)
            },
            priority: if req.priority == 0 {
                None
            } else {
                Some(req.priority as u32)
            },
            custom_fields: req.custom_fields,
            draft: req.draft,
        };

        let issue_id = match resolve_issue_id(project_path, &req.issue_id).await {
            Ok(id) => id,
            Err(e) => {
                return Ok(Response::new(UpdateIssueResponse {
                    success: false,
                    error: e,
                    ..Default::default()
                }))
            }
        };

        match update_issue(project_path, &issue_id, options).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Issue,
                    HookOperation::Update,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                // Convert sync results to proto
                let sync_results: Vec<OrgDocSyncResult> = result
                    .sync_results
                    .into_iter()
                    .map(|r| OrgDocSyncResult {
                        project_path: r.project_path,
                        success: r.success,
                        error: r.error.unwrap_or_default(),
                    })
                    .collect();

                Ok(Response::new(UpdateIssueResponse {
                    success: true,
                    error: String::new(),
                    issue: Some(issue_to_proto(&result.issue, priority_levels)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                    sync_results,
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Issue,
                    HookOperation::Update,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(UpdateIssueResponse {
                    success: false,
                    error: e.to_string(),
                    issue: None,
                    manifest: None,
                    sync_results: vec![],
                }))
            }
        }
    }

    async fn delete_issue(
        &self,
        request: Request<DeleteIssueRequest>,
    ) -> Result<Response<DeleteIssueResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.issue_id.clone();
        let hook_request_data = serde_json::json!({
            "issue_id": &req.issue_id,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Issue,
            HookOperation::Delete,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(DeleteIssueResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        let issue_id = match resolve_issue_id(project_path, &req.issue_id).await {
            Ok(id) => id,
            Err(e) => {
                return Ok(Response::new(DeleteIssueResponse {
                    success: false,
                    error: e,
                    ..Default::default()
                }))
            }
        };

        match delete_issue(project_path, &issue_id).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Issue,
                    HookOperation::Delete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(DeleteIssueResponse {
                    success: true,
                    error: String::new(),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Issue,
                    HookOperation::Delete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(DeleteIssueResponse {
                    success: false,
                    error: e.to_string(),
                    manifest: None,
                }))
            }
        }
    }

    async fn soft_delete_issue(
        &self,
        request: Request<SoftDeleteIssueRequest>,
    ) -> Result<Response<SoftDeleteIssueResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.issue_id.clone();
        let hook_request_data = serde_json::json!({
            "issue_id": &req.issue_id,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Issue,
            HookOperation::SoftDelete,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(SoftDeleteIssueResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        // Read config for priority_levels
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        let issue_id = match resolve_issue_id(project_path, &req.issue_id).await {
            Ok(id) => id,
            Err(e) => {
                return Ok(Response::new(SoftDeleteIssueResponse {
                    success: false,
                    error: e,
                    ..Default::default()
                }))
            }
        };

        match soft_delete_issue(project_path, &issue_id).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Issue,
                    HookOperation::SoftDelete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(SoftDeleteIssueResponse {
                    success: true,
                    error: String::new(),
                    issue: Some(issue_to_proto(&result.issue, priority_levels)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Issue,
                    HookOperation::SoftDelete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(SoftDeleteIssueResponse {
                    success: false,
                    error: e.to_string(),
                    issue: None,
                    manifest: None,
                }))
            }
        }
    }

    async fn restore_issue(
        &self,
        request: Request<RestoreIssueRequest>,
    ) -> Result<Response<RestoreIssueResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.issue_id.clone();
        let hook_request_data = serde_json::json!({
            "issue_id": &req.issue_id,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Issue,
            HookOperation::Restore,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(RestoreIssueResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        // Read config for priority_levels
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        let issue_id = match resolve_issue_id(project_path, &req.issue_id).await {
            Ok(id) => id,
            Err(e) => {
                return Ok(Response::new(RestoreIssueResponse {
                    success: false,
                    error: e,
                    ..Default::default()
                }))
            }
        };

        match restore_issue(project_path, &issue_id).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Issue,
                    HookOperation::Restore,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(RestoreIssueResponse {
                    success: true,
                    error: String::new(),
                    issue: Some(issue_to_proto(&result.issue, priority_levels)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Issue,
                    HookOperation::Restore,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(RestoreIssueResponse {
                    success: false,
                    error: e.to_string(),
                    issue: None,
                    manifest: None,
                }))
            }
        }
    }

    async fn move_issue(
        &self,
        request: Request<MoveIssueRequest>,
    ) -> Result<Response<MoveIssueResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.source_project_path.clone());
        track_project_async(req.target_project_path.clone());

        // Pre-hook
        let hook_project_path = req.source_project_path.clone();
        let hook_item_id = req.issue_id.clone();
        let hook_request_data = serde_json::json!({
            "source_project_path": &req.source_project_path,
            "target_project_path": &req.target_project_path,
            "issue_id": &req.issue_id,
        });
        if let Err(e) = maybe_run_pre_hooks(
            Path::new(&hook_project_path),
            HookItemType::Issue,
            HookOperation::Move,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(MoveIssueResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        // Read target config for priority_levels
        let target_config = read_config(Path::new(&req.target_project_path))
            .await
            .ok()
            .flatten();
        let priority_levels = target_config.as_ref().map_or(3, |c| c.priority_levels);

        let options = MoveIssueOptions {
            source_project_path: PathBuf::from(&req.source_project_path),
            target_project_path: PathBuf::from(&req.target_project_path),
            issue_id: req.issue_id,
        };

        match move_issue(options).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    Path::new(&hook_project_path),
                    HookItemType::Issue,
                    HookOperation::Move,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(MoveIssueResponse {
                    success: true,
                    error: String::new(),
                    issue: Some(issue_to_proto(&result.issue, priority_levels)),
                    old_display_number: result.old_display_number,
                    source_manifest: Some(manifest_to_proto(&result.source_manifest)),
                    target_manifest: Some(manifest_to_proto(&result.target_manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    Path::new(&hook_project_path),
                    HookItemType::Issue,
                    HookOperation::Move,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(MoveIssueResponse {
                    success: false,
                    error: e.to_string(),
                    issue: None,
                    old_display_number: 0,
                    source_manifest: None,
                    target_manifest: None,
                }))
            }
        }
    }

    async fn duplicate_issue(
        &self,
        request: Request<DuplicateIssueRequest>,
    ) -> Result<Response<DuplicateIssueResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.source_project_path.clone());
        track_project_async(req.target_project_path.clone());

        // Pre-hook
        let hook_project_path = req.source_project_path.clone();
        let hook_item_id = req.issue_id.clone();
        let hook_request_data = serde_json::json!({
            "source_project_path": &req.source_project_path,
            "target_project_path": &req.target_project_path,
            "issue_id": &req.issue_id,
        });
        if let Err(e) = maybe_run_pre_hooks(
            Path::new(&hook_project_path),
            HookItemType::Issue,
            HookOperation::Duplicate,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(DuplicateIssueResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        // Read target config for priority_levels
        let target_config = read_config(Path::new(&req.target_project_path))
            .await
            .ok()
            .flatten();
        let priority_levels = target_config.as_ref().map_or(3, |c| c.priority_levels);

        let options = DuplicateIssueOptions {
            source_project_path: PathBuf::from(&req.source_project_path),
            target_project_path: PathBuf::from(&req.target_project_path),
            issue_id: req.issue_id,
            new_title: if req.new_title.is_empty() {
                None
            } else {
                Some(req.new_title)
            },
        };

        match duplicate_issue(options).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    Path::new(&hook_project_path),
                    HookItemType::Issue,
                    HookOperation::Duplicate,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(DuplicateIssueResponse {
                    success: true,
                    error: String::new(),
                    issue: Some(issue_to_proto(&result.issue, priority_levels)),
                    original_issue_id: result.original_issue_id,
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    Path::new(&hook_project_path),
                    HookItemType::Issue,
                    HookOperation::Duplicate,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(DuplicateIssueResponse {
                    success: false,
                    error: e.to_string(),
                    issue: None,
                    original_issue_id: String::new(),
                    manifest: None,
                }))
            }
        }
    }

    async fn get_next_issue_number(
        &self,
        request: Request<GetNextIssueNumberRequest>,
    ) -> Result<Response<GetNextIssueNumberResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);
        let issues_path = get_centy_path(project_path).join("issues");

        #[allow(deprecated)]
        match crate::item::entities::issue::create::get_next_issue_number(&issues_path).await {
            Ok(issue_number) => Ok(Response::new(GetNextIssueNumberResponse { issue_number })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_manifest(
        &self,
        request: Request<GetManifestRequest>,
    ) -> Result<Response<GetManifestResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match read_manifest(project_path).await {
            Ok(Some(manifest)) => Ok(Response::new(GetManifestResponse {
                success: true,
                error: String::new(),
                manifest: Some(manifest_to_proto(&manifest)),
            })),
            Ok(None) => Ok(Response::new(GetManifestResponse {
                success: false,
                error: "Manifest not found".to_string(),
                manifest: None,
            })),
            Err(e) => Ok(Response::new(GetManifestResponse {
                success: false,
                error: e.to_string(),
                manifest: None,
            })),
        }
    }

    async fn get_config(
        &self,
        request: Request<GetConfigRequest>,
    ) -> Result<Response<GetConfigResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match read_config(project_path).await {
            Ok(Some(config)) => Ok(Response::new(GetConfigResponse {
                success: true,
                error: String::new(),
                config: Some(config_to_proto(&config)),
            })),
            Ok(None) => Ok(Response::new(GetConfigResponse {
                success: true,
                error: String::new(),
                config: Some(Config {
                    custom_fields: vec![],
                    defaults: std::collections::HashMap::new(),
                    priority_levels: 3, // Default
                    allowed_states: vec![
                        "open".to_string(),
                        "in-progress".to_string(),
                        "closed".to_string(),
                    ],
                    default_state: "open".to_string(),
                    version: crate::utils::CENTY_VERSION.to_string(),
                    state_colors: std::collections::HashMap::new(),
                    priority_colors: std::collections::HashMap::new(),
                    llm: Some(LlmConfig {
                        auto_close_on_complete: false,
                        update_status_on_start: None,
                        allow_direct_edits: false,
                        default_workspace_mode: 0,
                    }),
                    custom_link_types: vec![],
                    default_editor: String::new(),
                    hooks: vec![],
                }),
            })),
            Err(e) => Ok(Response::new(GetConfigResponse {
                success: false,
                error: e.to_string(),
                config: None,
            })),
        }
    }

    async fn update_config(
        &self,
        request: Request<UpdateConfigRequest>,
    ) -> Result<Response<UpdateConfigResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Check if project is initialized
        let centy_path = get_centy_path(project_path);
        let manifest_path = centy_path.join(".centy-manifest.json");
        if !manifest_path.exists() {
            return Ok(Response::new(UpdateConfigResponse {
                success: false,
                error: "Project not initialized".to_string(),
                config: None,
            }));
        }

        // Convert proto to internal config
        let proto_config = match req.config {
            Some(c) => c,
            None => {
                return Ok(Response::new(UpdateConfigResponse {
                    success: false,
                    error: "No config provided".to_string(),
                    config: None,
                }));
            }
        };
        let config = proto_to_config(&proto_config);

        // Validate config
        if let Err(e) = validate_config(&config) {
            return Ok(Response::new(UpdateConfigResponse {
                success: false,
                error: e,
                config: None,
            }));
        }

        // Write config
        match write_config(project_path, &config).await {
            Ok(()) => Ok(Response::new(UpdateConfigResponse {
                success: true,
                error: String::new(),
                config: Some(config_to_proto(&config)),
            })),
            Err(e) => Ok(Response::new(UpdateConfigResponse {
                success: false,
                error: e.to_string(),
                config: None,
            })),
        }
    }

    async fn is_initialized(
        &self,
        request: Request<IsInitializedRequest>,
    ) -> Result<Response<IsInitializedResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);
        let centy_path = get_centy_path(project_path);
        let manifest_path = centy_path.join(".centy-manifest.json");

        let initialized = manifest_path.exists();
        let centy_path_str = if initialized {
            centy_path.to_string_lossy().to_string()
        } else {
            String::new()
        };

        Ok(Response::new(IsInitializedResponse {
            initialized,
            centy_path: centy_path_str,
        }))
    }

    // ============ Doc RPCs ============

    async fn create_doc(
        &self,
        request: Request<CreateDocRequest>,
    ) -> Result<Response<CreateDocResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_request_data = serde_json::json!({
            "title": &req.title,
            "content": &req.content,
            "slug": &req.slug,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Doc,
            HookOperation::Create,
            &hook_project_path,
            None,
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(CreateDocResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        let options = CreateDocOptions {
            title: req.title,
            content: req.content,
            slug: if req.slug.is_empty() {
                None
            } else {
                Some(req.slug)
            },
            template: if req.template.is_empty() {
                None
            } else {
                Some(req.template)
            },
            is_org_doc: req.is_org_doc,
        };

        match create_doc(project_path, options).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Doc,
                    HookOperation::Create,
                    &hook_project_path,
                    Some(&result.slug),
                    Some(hook_request_data),
                    true,
                )
                .await;

                // Convert sync results to proto
                let sync_results: Vec<OrgDocSyncResult> = result
                    .sync_results
                    .into_iter()
                    .map(|r| OrgDocSyncResult {
                        project_path: r.project_path,
                        success: r.success,
                        error: r.error.unwrap_or_default(),
                    })
                    .collect();

                Ok(Response::new(CreateDocResponse {
                    success: true,
                    error: String::new(),
                    slug: result.slug,
                    created_file: result.created_file,
                    manifest: Some(manifest_to_proto(&result.manifest)),
                    sync_results,
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Doc,
                    HookOperation::Create,
                    &hook_project_path,
                    None,
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(CreateDocResponse {
                    success: false,
                    error: e.to_string(),
                    slug: String::new(),
                    created_file: String::new(),
                    manifest: None,
                    sync_results: Vec::new(),
                }))
            }
        }
    }

    async fn get_doc(
        &self,
        request: Request<GetDocRequest>,
    ) -> Result<Response<GetDocResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match get_doc(project_path, &req.slug).await {
            Ok(doc) => Ok(Response::new(GetDocResponse {
                success: true,
                error: String::new(),
                doc: Some(doc_to_proto(&doc)),
            })),
            Err(e) => Ok(Response::new(GetDocResponse {
                success: false,
                error: e.to_string(),
                doc: None,
            })),
        }
    }

    async fn get_docs_by_slug(
        &self,
        request: Request<GetDocsBySlugRequest>,
    ) -> Result<Response<GetDocsBySlugResponse>, Status> {
        let req = request.into_inner();

        // Get all initialized projects from registry
        let projects = match list_projects(ListProjectsOptions::default()).await {
            Ok(p) => p,
            Err(e) => return Err(Status::internal(format!("Failed to list projects: {e}"))),
        };

        match get_docs_by_slug(&req.slug, &projects).await {
            Ok(result) => {
                let docs_with_projects: Vec<ProtoDocWithProject> = result
                    .docs
                    .into_iter()
                    .map(|dwp| ProtoDocWithProject {
                        doc: Some(doc_to_proto(&dwp.doc)),
                        display_path: format_display_path(&dwp.project_path),
                        project_path: dwp.project_path,
                        project_name: dwp.project_name,
                    })
                    .collect();

                let total_count = docs_with_projects.len() as i32;

                Ok(Response::new(GetDocsBySlugResponse {
                    docs: docs_with_projects,
                    total_count,
                    errors: result.errors,
                }))
            }
            Err(e) => Err(Status::invalid_argument(e.to_string())),
        }
    }

    async fn list_docs(
        &self,
        request: Request<ListDocsRequest>,
    ) -> Result<Response<ListDocsResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match list_docs(project_path, false).await {
            Ok(docs) => {
                let total_count = docs.len() as i32;
                Ok(Response::new(ListDocsResponse {
                    docs: docs.into_iter().map(|d| doc_to_proto(&d)).collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_doc(
        &self,
        request: Request<UpdateDocRequest>,
    ) -> Result<Response<UpdateDocResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.slug.clone();
        let hook_request_data = serde_json::json!({
            "slug": &req.slug,
            "title": &req.title,
            "content": &req.content,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Doc,
            HookOperation::Update,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(UpdateDocResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        let options = UpdateDocOptions {
            title: if req.title.is_empty() {
                None
            } else {
                Some(req.title)
            },
            content: if req.content.is_empty() {
                None
            } else {
                Some(req.content)
            },
            new_slug: if req.new_slug.is_empty() {
                None
            } else {
                Some(req.new_slug)
            },
        };

        match update_doc(project_path, &req.slug, options).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Doc,
                    HookOperation::Update,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                let sync_results: Vec<OrgDocSyncResult> = result
                    .sync_results
                    .into_iter()
                    .map(|r| OrgDocSyncResult {
                        project_path: r.project_path,
                        success: r.success,
                        error: r.error.unwrap_or_default(),
                    })
                    .collect();

                Ok(Response::new(UpdateDocResponse {
                    success: true,
                    error: String::new(),
                    doc: Some(doc_to_proto(&result.doc)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                    sync_results,
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Doc,
                    HookOperation::Update,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(UpdateDocResponse {
                    success: false,
                    error: e.to_string(),
                    doc: None,
                    manifest: None,
                    sync_results: Vec::new(),
                }))
            }
        }
    }

    async fn delete_doc(
        &self,
        request: Request<DeleteDocRequest>,
    ) -> Result<Response<DeleteDocResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.slug.clone();
        let hook_request_data = serde_json::json!({
            "slug": &req.slug,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Doc,
            HookOperation::Delete,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(DeleteDocResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        match delete_doc(project_path, &req.slug).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Doc,
                    HookOperation::Delete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(DeleteDocResponse {
                    success: true,
                    error: String::new(),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Doc,
                    HookOperation::Delete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(DeleteDocResponse {
                    success: false,
                    error: e.to_string(),
                    manifest: None,
                }))
            }
        }
    }

    async fn soft_delete_doc(
        &self,
        request: Request<SoftDeleteDocRequest>,
    ) -> Result<Response<SoftDeleteDocResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.slug.clone();
        let hook_request_data = serde_json::json!({
            "slug": &req.slug,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Doc,
            HookOperation::SoftDelete,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(SoftDeleteDocResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        match soft_delete_doc(project_path, &req.slug).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Doc,
                    HookOperation::SoftDelete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(SoftDeleteDocResponse {
                    success: true,
                    error: String::new(),
                    doc: Some(doc_to_proto(&result.doc)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Doc,
                    HookOperation::SoftDelete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(SoftDeleteDocResponse {
                    success: false,
                    error: e.to_string(),
                    doc: None,
                    manifest: None,
                }))
            }
        }
    }

    async fn restore_doc(
        &self,
        request: Request<RestoreDocRequest>,
    ) -> Result<Response<RestoreDocResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.slug.clone();
        let hook_request_data = serde_json::json!({
            "slug": &req.slug,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Doc,
            HookOperation::Restore,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(RestoreDocResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        match restore_doc(project_path, &req.slug).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Doc,
                    HookOperation::Restore,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(RestoreDocResponse {
                    success: true,
                    error: String::new(),
                    doc: Some(doc_to_proto(&result.doc)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Doc,
                    HookOperation::Restore,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(RestoreDocResponse {
                    success: false,
                    error: e.to_string(),
                    doc: None,
                    manifest: None,
                }))
            }
        }
    }

    async fn move_doc(
        &self,
        request: Request<MoveDocRequest>,
    ) -> Result<Response<MoveDocResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.source_project_path.clone());
        track_project_async(req.target_project_path.clone());

        // Pre-hook
        let hook_project_path = req.source_project_path.clone();
        let hook_item_id = req.slug.clone();
        let hook_request_data = serde_json::json!({
            "source_project_path": &req.source_project_path,
            "target_project_path": &req.target_project_path,
            "slug": &req.slug,
        });
        if let Err(e) = maybe_run_pre_hooks(
            Path::new(&hook_project_path),
            HookItemType::Doc,
            HookOperation::Move,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(MoveDocResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        let options = MoveDocOptions {
            source_project_path: PathBuf::from(&req.source_project_path),
            target_project_path: PathBuf::from(&req.target_project_path),
            slug: req.slug.clone(),
            new_slug: if req.new_slug.is_empty() {
                None
            } else {
                Some(req.new_slug)
            },
        };

        match move_doc(options).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    Path::new(&hook_project_path),
                    HookItemType::Doc,
                    HookOperation::Move,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(MoveDocResponse {
                    success: true,
                    error: String::new(),
                    doc: Some(doc_to_proto(&result.doc)),
                    old_slug: result.old_slug,
                    source_manifest: Some(manifest_to_proto(&result.source_manifest)),
                    target_manifest: Some(manifest_to_proto(&result.target_manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    Path::new(&hook_project_path),
                    HookItemType::Doc,
                    HookOperation::Move,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(MoveDocResponse {
                    success: false,
                    error: e.to_string(),
                    doc: None,
                    old_slug: req.slug,
                    source_manifest: None,
                    target_manifest: None,
                }))
            }
        }
    }

    async fn duplicate_doc(
        &self,
        request: Request<DuplicateDocRequest>,
    ) -> Result<Response<DuplicateDocResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.source_project_path.clone());
        track_project_async(req.target_project_path.clone());

        // Pre-hook
        let hook_project_path = req.source_project_path.clone();
        let hook_item_id = req.slug.clone();
        let hook_request_data = serde_json::json!({
            "source_project_path": &req.source_project_path,
            "target_project_path": &req.target_project_path,
            "slug": &req.slug,
        });
        if let Err(e) = maybe_run_pre_hooks(
            Path::new(&hook_project_path),
            HookItemType::Doc,
            HookOperation::Duplicate,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(DuplicateDocResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        let options = DuplicateDocOptions {
            source_project_path: PathBuf::from(&req.source_project_path),
            target_project_path: PathBuf::from(&req.target_project_path),
            slug: req.slug.clone(),
            new_slug: if req.new_slug.is_empty() {
                None
            } else {
                Some(req.new_slug)
            },
            new_title: if req.new_title.is_empty() {
                None
            } else {
                Some(req.new_title)
            },
        };

        match duplicate_doc(options).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    Path::new(&hook_project_path),
                    HookItemType::Doc,
                    HookOperation::Duplicate,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(DuplicateDocResponse {
                    success: true,
                    error: String::new(),
                    doc: Some(doc_to_proto(&result.doc)),
                    original_slug: result.original_slug,
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    Path::new(&hook_project_path),
                    HookItemType::Doc,
                    HookOperation::Duplicate,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(DuplicateDocResponse {
                    success: false,
                    error: e.to_string(),
                    doc: None,
                    original_slug: req.slug,
                    manifest: None,
                }))
            }
        }
    }

    // ============ Asset RPCs ============

    async fn add_asset(
        &self,
        request: Request<AddAssetRequest>,
    ) -> Result<Response<AddAssetResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.filename.clone();
        let hook_request_data = serde_json::json!({
            "filename": &req.filename,
            "issue_id": &req.issue_id,
            "is_shared": req.is_shared,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Asset,
            HookOperation::Create,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(AddAssetResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        let scope = if req.is_shared {
            AssetScope::Shared
        } else {
            AssetScope::IssueSpecific
        };

        let issue_id = if req.issue_id.is_empty() {
            None
        } else {
            Some(req.issue_id.as_str())
        };

        match add_asset(project_path, issue_id, req.data, &req.filename, scope).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Asset,
                    HookOperation::Create,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                // Re-read manifest for response
                let manifest = read_manifest(project_path).await.ok().flatten();
                Ok(Response::new(AddAssetResponse {
                    success: true,
                    error: String::new(),
                    asset: Some(asset_info_to_proto(&result.asset)),
                    path: result.path,
                    manifest: manifest.map(|m| manifest_to_proto(&m)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Asset,
                    HookOperation::Create,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(AddAssetResponse {
                    success: false,
                    error: e.to_string(),
                    asset: None,
                    path: String::new(),
                    manifest: None,
                }))
            }
        }
    }

    async fn list_assets(
        &self,
        request: Request<ListAssetsRequest>,
    ) -> Result<Response<ListAssetsResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match list_assets(project_path, &req.issue_id, req.include_shared).await {
            Ok(assets) => {
                let total_count = assets.len() as i32;
                Ok(Response::new(ListAssetsResponse {
                    assets: assets.iter().map(asset_info_to_proto).collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_asset(
        &self,
        request: Request<GetAssetRequest>,
    ) -> Result<Response<GetAssetResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let issue_id = if req.issue_id.is_empty() {
            None
        } else {
            Some(req.issue_id.as_str())
        };

        match get_asset(project_path, issue_id, &req.filename, req.is_shared).await {
            Ok((data, asset_info)) => Ok(Response::new(GetAssetResponse {
                success: true,
                error: String::new(),
                data,
                asset: Some(asset_info_to_proto(&asset_info)),
            })),
            Err(e) => Ok(Response::new(GetAssetResponse {
                success: false,
                error: e.to_string(),
                data: vec![],
                asset: None,
            })),
        }
    }

    async fn delete_asset(
        &self,
        request: Request<DeleteAssetRequest>,
    ) -> Result<Response<DeleteAssetResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.filename.clone();
        let hook_request_data = serde_json::json!({
            "filename": &req.filename,
            "issue_id": &req.issue_id,
            "is_shared": req.is_shared,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Asset,
            HookOperation::Delete,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(DeleteAssetResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        let issue_id = if req.issue_id.is_empty() {
            None
        } else {
            Some(req.issue_id.as_str())
        };

        match delete_asset_fn(project_path, issue_id, &req.filename, req.is_shared).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Asset,
                    HookOperation::Delete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                // Re-read manifest for response
                let manifest = read_manifest(project_path).await.ok().flatten();
                Ok(Response::new(DeleteAssetResponse {
                    success: true,
                    error: String::new(),
                    filename: result.filename,
                    was_shared: result.was_shared,
                    manifest: manifest.map(|m| manifest_to_proto(&m)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Asset,
                    HookOperation::Delete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(DeleteAssetResponse {
                    success: false,
                    error: e.to_string(),
                    filename: String::new(),
                    was_shared: false,
                    manifest: None,
                }))
            }
        }
    }

    async fn list_shared_assets(
        &self,
        request: Request<ListSharedAssetsRequest>,
    ) -> Result<Response<ListAssetsResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match list_shared_assets(project_path).await {
            Ok(assets) => {
                let total_count = assets.len() as i32;
                Ok(Response::new(ListAssetsResponse {
                    assets: assets.iter().map(asset_info_to_proto).collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // ============ Project Registry RPCs ============

    async fn list_projects(
        &self,
        request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let req = request.into_inner();

        let org_slug = if req.organization_slug.is_empty() {
            None
        } else {
            Some(req.organization_slug.as_str())
        };
        let opts = ListProjectsOptions {
            include_stale: req.include_stale,
            include_uninitialized: req.include_uninitialized,
            include_archived: req.include_archived,
            organization_slug: org_slug,
            ungrouped_only: req.ungrouped_only,
            include_temp: req.include_temp,
        };
        match list_projects(opts).await {
            Ok(projects) => {
                let total_count = projects.len() as i32;
                Ok(Response::new(ListProjectsResponse {
                    projects: projects
                        .into_iter()
                        .map(|p| project_info_to_proto(&p))
                        .collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn register_project(
        &self,
        request: Request<RegisterProjectRequest>,
    ) -> Result<Response<RegisterProjectResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        // Track the project (this creates or updates the entry)
        if let Err(e) = crate::registry::track_project(&req.project_path).await {
            return Ok(Response::new(RegisterProjectResponse {
                success: false,
                error: e.to_string(),
                project: None,
                org_inference: None,
            }));
        }

        // Infer organization from git remote
        let existing_org = get_project_info(&req.project_path)
            .await
            .ok()
            .flatten()
            .and_then(|info| info.organization_slug);
        let inference = infer_organization_from_remote(project_path, existing_org.as_deref()).await;

        // Auto-assign if no existing org and inference succeeded without mismatch
        if existing_org.is_none() && !inference.has_mismatch {
            if let Some(ref slug) = inference.inferred_org_slug {
                let _ = set_project_organization(&req.project_path, Some(slug)).await;
            }
        }

        // Get the project info (refresh after potential org assignment)
        match get_project_info(&req.project_path).await {
            Ok(Some(info)) => Ok(Response::new(RegisterProjectResponse {
                success: true,
                error: String::new(),
                project: Some(project_info_to_proto(&info)),
                org_inference: Some(org_inference_to_proto(&inference)),
            })),
            Ok(None) => Ok(Response::new(RegisterProjectResponse {
                success: false,
                error: "Failed to retrieve project after registration".to_string(),
                project: None,
                org_inference: Some(org_inference_to_proto(&inference)),
            })),
            Err(e) => Ok(Response::new(RegisterProjectResponse {
                success: false,
                error: e.to_string(),
                project: None,
                org_inference: Some(org_inference_to_proto(&inference)),
            })),
        }
    }

    async fn untrack_project(
        &self,
        request: Request<UntrackProjectRequest>,
    ) -> Result<Response<UntrackProjectResponse>, Status> {
        let req = request.into_inner();

        match untrack_project(&req.project_path).await {
            Ok(()) => Ok(Response::new(UntrackProjectResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(UntrackProjectResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn get_project_info(
        &self,
        request: Request<GetProjectInfoRequest>,
    ) -> Result<Response<GetProjectInfoResponse>, Status> {
        let req = request.into_inner();

        match get_project_info(&req.project_path).await {
            Ok(Some(info)) => Ok(Response::new(GetProjectInfoResponse {
                found: true,
                project: Some(project_info_to_proto(&info)),
            })),
            Ok(None) => Ok(Response::new(GetProjectInfoResponse {
                found: false,
                project: None,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn set_project_favorite(
        &self,
        request: Request<SetProjectFavoriteRequest>,
    ) -> Result<Response<SetProjectFavoriteResponse>, Status> {
        let req = request.into_inner();

        match set_project_favorite(&req.project_path, req.is_favorite).await {
            Ok(info) => Ok(Response::new(SetProjectFavoriteResponse {
                success: true,
                error: String::new(),
                project: Some(project_info_to_proto(&info)),
            })),
            Err(e) => Ok(Response::new(SetProjectFavoriteResponse {
                success: false,
                error: e.to_string(),
                project: None,
            })),
        }
    }

    async fn set_project_archived(
        &self,
        request: Request<SetProjectArchivedRequest>,
    ) -> Result<Response<SetProjectArchivedResponse>, Status> {
        let req = request.into_inner();

        match set_project_archived(&req.project_path, req.is_archived).await {
            Ok(info) => Ok(Response::new(SetProjectArchivedResponse {
                success: true,
                error: String::new(),
                project: Some(project_info_to_proto(&info)),
            })),
            Err(e) => Ok(Response::new(SetProjectArchivedResponse {
                success: false,
                error: e.to_string(),
                project: None,
            })),
        }
    }

    async fn set_project_organization(
        &self,
        request: Request<SetProjectOrganizationRequest>,
    ) -> Result<Response<SetProjectOrganizationResponse>, Status> {
        let req = request.into_inner();
        let org_slug = if req.organization_slug.is_empty() {
            None
        } else {
            Some(req.organization_slug.as_str())
        };

        match set_project_organization(&req.project_path, org_slug).await {
            Ok(info) => Ok(Response::new(SetProjectOrganizationResponse {
                success: true,
                error: String::new(),
                project: Some(project_info_to_proto(&info)),
            })),
            Err(e) => Ok(Response::new(SetProjectOrganizationResponse {
                success: false,
                error: e.to_string(),
                project: None,
            })),
        }
    }

    async fn set_project_user_title(
        &self,
        request: Request<SetProjectUserTitleRequest>,
    ) -> Result<Response<SetProjectUserTitleResponse>, Status> {
        let req = request.into_inner();
        let title = if req.title.is_empty() {
            None
        } else {
            Some(req.title)
        };

        match set_project_user_title(&req.project_path, title).await {
            Ok(info) => Ok(Response::new(SetProjectUserTitleResponse {
                success: true,
                error: String::new(),
                project: Some(project_info_to_proto(&info)),
            })),
            Err(e) => Ok(Response::new(SetProjectUserTitleResponse {
                success: false,
                error: e.to_string(),
                project: None,
            })),
        }
    }

    async fn set_project_title(
        &self,
        request: Request<SetProjectTitleRequest>,
    ) -> Result<Response<SetProjectTitleResponse>, Status> {
        let req = request.into_inner();
        let title = if req.title.is_empty() {
            None
        } else {
            Some(req.title)
        };
        let project_path = Path::new(&req.project_path);

        // Set project-scope title in .centy/project.json
        match set_project_title_config(project_path, title).await {
            Ok(()) => {
                // Fetch updated project info
                match get_project_info(&req.project_path).await {
                    Ok(Some(info)) => Ok(Response::new(SetProjectTitleResponse {
                        success: true,
                        error: String::new(),
                        project: Some(project_info_to_proto(&info)),
                    })),
                    Ok(None) => Ok(Response::new(SetProjectTitleResponse {
                        success: false,
                        error: "Project not found in registry".to_string(),
                        project: None,
                    })),
                    Err(e) => Ok(Response::new(SetProjectTitleResponse {
                        success: false,
                        error: e.to_string(),
                        project: None,
                    })),
                }
            }
            Err(e) => Ok(Response::new(SetProjectTitleResponse {
                success: false,
                error: e.to_string(),
                project: None,
            })),
        }
    }

    // ============ Organization RPCs ============

    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<CreateOrganizationResponse>, Status> {
        let req = request.into_inner();
        let slug = if req.slug.is_empty() {
            None
        } else {
            Some(req.slug.as_str())
        };
        let description = if req.description.is_empty() {
            None
        } else {
            Some(req.description.as_str())
        };

        match create_organization(slug, &req.name, description).await {
            Ok(org) => Ok(Response::new(CreateOrganizationResponse {
                success: true,
                error: String::new(),
                organization: Some(org_info_to_proto(&org)),
            })),
            Err(e) => Ok(Response::new(CreateOrganizationResponse {
                success: false,
                error: e.to_string(),
                organization: None,
            })),
        }
    }

    async fn list_organizations(
        &self,
        _request: Request<ListOrganizationsRequest>,
    ) -> Result<Response<ListOrganizationsResponse>, Status> {
        match list_organizations().await {
            Ok(orgs) => {
                let total_count = orgs.len() as i32;
                Ok(Response::new(ListOrganizationsResponse {
                    organizations: orgs.into_iter().map(|o| org_info_to_proto(&o)).collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_organization(
        &self,
        request: Request<GetOrganizationRequest>,
    ) -> Result<Response<GetOrganizationResponse>, Status> {
        let req = request.into_inner();

        match get_organization(&req.slug).await {
            Ok(Some(org)) => Ok(Response::new(GetOrganizationResponse {
                found: true,
                organization: Some(org_info_to_proto(&org)),
            })),
            Ok(None) => Ok(Response::new(GetOrganizationResponse {
                found: false,
                organization: None,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_organization(
        &self,
        request: Request<UpdateOrganizationRequest>,
    ) -> Result<Response<UpdateOrganizationResponse>, Status> {
        let req = request.into_inner();
        let name = if req.name.is_empty() {
            None
        } else {
            Some(req.name.as_str())
        };
        let description = if req.description.is_empty() {
            None
        } else {
            Some(req.description.as_str())
        };
        let new_slug = if req.new_slug.is_empty() {
            None
        } else {
            Some(req.new_slug.as_str())
        };

        match update_organization(&req.slug, name, description, new_slug).await {
            Ok(org) => Ok(Response::new(UpdateOrganizationResponse {
                success: true,
                error: String::new(),
                organization: Some(org_info_to_proto(&org)),
            })),
            Err(e) => Ok(Response::new(UpdateOrganizationResponse {
                success: false,
                error: e.to_string(),
                organization: None,
            })),
        }
    }

    async fn delete_organization(
        &self,
        request: Request<DeleteOrganizationRequest>,
    ) -> Result<Response<DeleteOrganizationResponse>, Status> {
        let req = request.into_inner();

        match delete_organization(&req.slug).await {
            Ok(()) => Ok(Response::new(DeleteOrganizationResponse {
                success: true,
                error: String::new(),
                unassigned_projects: 0,
            })),
            Err(e) => Ok(Response::new(DeleteOrganizationResponse {
                success: false,
                error: e.to_string(),
                unassigned_projects: 0,
            })),
        }
    }

    // ============ Version RPCs ============

    async fn get_daemon_info(
        &self,
        _request: Request<GetDaemonInfoRequest>,
    ) -> Result<Response<DaemonInfo>, Status> {
        let binary_path = std::env::current_exe()
            .map(|p| format_display_path(&p.to_string_lossy()))
            .unwrap_or_default();

        Ok(Response::new(DaemonInfo {
            version: CENTY_VERSION.to_string(),
            binary_path,
            vscode_available: is_vscode_available(),
        }))
    }

    // ============ Daemon Control RPCs ============

    async fn shutdown(
        &self,
        request: Request<ShutdownRequest>,
    ) -> Result<Response<ShutdownResponse>, Status> {
        let req = request.into_inner();
        let delay = req.delay_seconds;

        info!("Shutdown requested with delay: {} seconds", delay);

        // Clone the sender for use in the spawned task
        let shutdown_tx = self.shutdown_tx.clone();

        // Spawn a task to handle the delayed shutdown
        // Always wait a small amount of time to ensure the response is sent before shutting down
        tokio::spawn(async move {
            if delay > 0 {
                tokio::time::sleep(tokio::time::Duration::from_secs(u64::from(delay))).await;
            } else {
                // Small delay to ensure the RPC response is fully sent before shutdown
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            let _ = shutdown_tx.send(ShutdownSignal::Shutdown);
        });

        let message = if delay > 0 {
            format!("Daemon will shutdown in {delay} seconds")
        } else {
            "Daemon shutting down".to_string()
        };

        Ok(Response::new(ShutdownResponse {
            success: true,
            message,
        }))
    }

    async fn restart(
        &self,
        request: Request<RestartRequest>,
    ) -> Result<Response<RestartResponse>, Status> {
        let req = request.into_inner();
        let delay = req.delay_seconds;

        info!("Restart requested with delay: {} seconds", delay);

        // Check if we have the executable path
        let exe_path = match &self.exe_path {
            Some(path) => path.clone(),
            None => {
                return Ok(Response::new(RestartResponse {
                    success: false,
                    message: "Cannot restart: unable to determine executable path".to_string(),
                }));
            }
        };

        // Clone what we need for the spawned task
        let shutdown_tx = self.shutdown_tx.clone();

        // Spawn a task to handle the delayed restart
        // Always wait a small amount of time to ensure the response is sent before restarting
        tokio::spawn(async move {
            if delay > 0 {
                tokio::time::sleep(tokio::time::Duration::from_secs(u64::from(delay))).await;
            } else {
                // Small delay to ensure the RPC response is fully sent before restart
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            // Spawn a new daemon process before shutting down
            info!("Spawning new daemon process: {:?}", exe_path);
            match Command::new(&exe_path).spawn() {
                Ok(_) => {
                    info!("New daemon process spawned successfully");
                    // Signal the current server to shutdown
                    let _ = shutdown_tx.send(ShutdownSignal::Restart);
                }
                Err(e) => {
                    info!("Failed to spawn new daemon process: {}", e);
                }
            }
        });

        let message = if delay > 0 {
            format!("Daemon will restart in {delay} seconds")
        } else {
            "Daemon restarting".to_string()
        };

        Ok(Response::new(RestartResponse {
            success: true,
            message,
        }))
    }

    // ============ PR RPCs ============

    async fn create_pr(
        &self,
        request: Request<CreatePrRequest>,
    ) -> Result<Response<CreatePrResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_request_data = serde_json::json!({
            "title": &req.title,
            "description": &req.description,
            "source_branch": &req.source_branch,
            "target_branch": &req.target_branch,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Pr,
            HookOperation::Create,
            &hook_project_path,
            None,
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(CreatePrResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        let options = CreatePrOptions {
            title: req.title,
            description: req.description,
            source_branch: if req.source_branch.is_empty() {
                None
            } else {
                Some(req.source_branch)
            },
            target_branch: if req.target_branch.is_empty() {
                None
            } else {
                Some(req.target_branch)
            },
            reviewers: req.reviewers,
            priority: if req.priority == 0 {
                None
            } else {
                Some(req.priority as u32)
            },
            status: if req.status.is_empty() {
                None
            } else {
                Some(req.status)
            },
            custom_fields: req.custom_fields,
            template: if req.template.is_empty() {
                None
            } else {
                Some(req.template)
            },
        };

        match create_pr(project_path, options).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Pr,
                    HookOperation::Create,
                    &hook_project_path,
                    Some(&result.id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(CreatePrResponse {
                    success: true,
                    error: String::new(),
                    id: result.id,
                    display_number: result.display_number,
                    created_files: result.created_files,
                    manifest: Some(manifest_to_proto(&result.manifest)),
                    detected_source_branch: result.detected_source_branch,
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Pr,
                    HookOperation::Create,
                    &hook_project_path,
                    None,
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(CreatePrResponse {
                    success: false,
                    error: e.to_string(),
                    id: String::new(),
                    display_number: 0,
                    created_files: vec![],
                    manifest: None,
                    detected_source_branch: String::new(),
                }))
            }
        }
    }

    async fn get_pr(
        &self,
        request: Request<GetPrRequest>,
    ) -> Result<Response<GetPrResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        match resolve_pr(project_path, &req.pr_id).await {
            Ok(pr) => Ok(Response::new(GetPrResponse {
                success: true,
                error: String::new(),
                pr: Some(pr_to_proto(&pr, priority_levels)),
            })),
            Err(e) => Ok(Response::new(GetPrResponse {
                success: false,
                error: e,
                pr: None,
            })),
        }
    }

    async fn get_pr_by_display_number(
        &self,
        request: Request<GetPrByDisplayNumberRequest>,
    ) -> Result<Response<GetPrResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        match get_pr_by_display_number(project_path, req.display_number).await {
            Ok(pr) => Ok(Response::new(GetPrResponse {
                success: true,
                error: String::new(),
                pr: Some(pr_to_proto(&pr, priority_levels)),
            })),
            Err(e) => Ok(Response::new(GetPrResponse {
                success: false,
                error: e.to_string(),
                pr: None,
            })),
        }
    }

    async fn get_prs_by_uuid(
        &self,
        request: Request<GetPrsByUuidRequest>,
    ) -> Result<Response<GetPrsByUuidResponse>, Status> {
        let req = request.into_inner();

        // Get all initialized projects from registry
        let projects = match list_projects(ListProjectsOptions::default()).await {
            Ok(p) => p,
            Err(e) => return Err(Status::internal(format!("Failed to list projects: {e}"))),
        };

        match get_prs_by_uuid(&req.uuid, &projects).await {
            Ok(result) => {
                let prs_with_projects: Vec<ProtoPrWithProject> = result
                    .prs
                    .into_iter()
                    .map(|pwp| {
                        // Use default priority_levels of 3 for global search
                        let priority_levels = 3;

                        ProtoPrWithProject {
                            pr: Some(pr_to_proto(&pwp.pr, priority_levels)),
                            display_path: format_display_path(&pwp.project_path),
                            project_path: pwp.project_path,
                            project_name: pwp.project_name,
                        }
                    })
                    .collect();

                let total_count = prs_with_projects.len() as i32;

                Ok(Response::new(GetPrsByUuidResponse {
                    prs: prs_with_projects,
                    total_count,
                    errors: result.errors,
                }))
            }
            Err(e) => Err(Status::invalid_argument(e.to_string())),
        }
    }

    async fn list_prs(
        &self,
        request: Request<ListPrsRequest>,
    ) -> Result<Response<ListPrsResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        let status_filter = if req.status.is_empty() {
            None
        } else {
            Some(req.status.as_str())
        };
        let source_filter = if req.source_branch.is_empty() {
            None
        } else {
            Some(req.source_branch.as_str())
        };
        let target_filter = if req.target_branch.is_empty() {
            None
        } else {
            Some(req.target_branch.as_str())
        };
        let priority_filter = if req.priority == 0 {
            None
        } else {
            Some(req.priority as u32)
        };

        match list_prs(
            project_path,
            status_filter,
            source_filter,
            target_filter,
            priority_filter,
            false,
        )
        .await
        {
            Ok(prs) => {
                let total_count = prs.len() as i32;
                Ok(Response::new(ListPrsResponse {
                    prs: prs
                        .into_iter()
                        .map(|p| pr_to_proto(&p, priority_levels))
                        .collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_pr(
        &self,
        request: Request<UpdatePrRequest>,
    ) -> Result<Response<UpdatePrResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.pr_id.clone();
        let hook_request_data = serde_json::json!({
            "pr_id": &req.pr_id,
            "title": &req.title,
            "description": &req.description,
            "status": &req.status,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Pr,
            HookOperation::Update,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(UpdatePrResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        let options = UpdatePrOptions {
            title: if req.title.is_empty() {
                None
            } else {
                Some(req.title)
            },
            description: if req.description.is_empty() {
                None
            } else {
                Some(req.description)
            },
            status: if req.status.is_empty() {
                None
            } else {
                Some(req.status)
            },
            source_branch: if req.source_branch.is_empty() {
                None
            } else {
                Some(req.source_branch)
            },
            target_branch: if req.target_branch.is_empty() {
                None
            } else {
                Some(req.target_branch)
            },
            reviewers: if req.reviewers.is_empty() {
                None
            } else {
                Some(req.reviewers)
            },
            priority: if req.priority == 0 {
                None
            } else {
                Some(req.priority as u32)
            },
            custom_fields: req.custom_fields,
        };

        let pr_id = match resolve_pr_id(project_path, &req.pr_id).await {
            Ok(id) => id,
            Err(e) => {
                return Ok(Response::new(UpdatePrResponse {
                    success: false,
                    error: e,
                    ..Default::default()
                }))
            }
        };

        match update_pr(project_path, &pr_id, options).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Pr,
                    HookOperation::Update,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(UpdatePrResponse {
                    success: true,
                    error: String::new(),
                    pr: Some(pr_to_proto(&result.pr, priority_levels)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Pr,
                    HookOperation::Update,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(UpdatePrResponse {
                    success: false,
                    error: e.to_string(),
                    pr: None,
                    manifest: None,
                }))
            }
        }
    }

    async fn delete_pr(
        &self,
        request: Request<DeletePrRequest>,
    ) -> Result<Response<DeletePrResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.pr_id.clone();
        let hook_request_data = serde_json::json!({
            "pr_id": &req.pr_id,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Pr,
            HookOperation::Delete,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(DeletePrResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        let pr_id = match resolve_pr_id(project_path, &req.pr_id).await {
            Ok(id) => id,
            Err(e) => {
                return Ok(Response::new(DeletePrResponse {
                    success: false,
                    error: e,
                    ..Default::default()
                }))
            }
        };

        match delete_pr(project_path, &pr_id).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Pr,
                    HookOperation::Delete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(DeletePrResponse {
                    success: true,
                    error: String::new(),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Pr,
                    HookOperation::Delete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(DeletePrResponse {
                    success: false,
                    error: e.to_string(),
                    manifest: None,
                }))
            }
        }
    }

    async fn soft_delete_pr(
        &self,
        request: Request<SoftDeletePrRequest>,
    ) -> Result<Response<SoftDeletePrResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.pr_id.clone();
        let hook_request_data = serde_json::json!({
            "pr_id": &req.pr_id,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Pr,
            HookOperation::SoftDelete,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(SoftDeletePrResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        // Read config for priority_levels
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        let pr_id = match resolve_pr_id(project_path, &req.pr_id).await {
            Ok(id) => id,
            Err(e) => {
                return Ok(Response::new(SoftDeletePrResponse {
                    success: false,
                    error: e,
                    ..Default::default()
                }))
            }
        };

        match soft_delete_pr(project_path, &pr_id).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Pr,
                    HookOperation::SoftDelete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(SoftDeletePrResponse {
                    success: true,
                    error: String::new(),
                    pr: Some(pr_to_proto(&result.pr, priority_levels)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Pr,
                    HookOperation::SoftDelete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(SoftDeletePrResponse {
                    success: false,
                    error: e.to_string(),
                    pr: None,
                    manifest: None,
                }))
            }
        }
    }

    async fn restore_pr(
        &self,
        request: Request<RestorePrRequest>,
    ) -> Result<Response<RestorePrResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.pr_id.clone();
        let hook_request_data = serde_json::json!({
            "pr_id": &req.pr_id,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Pr,
            HookOperation::Restore,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(RestorePrResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        // Read config for priority_levels
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        let pr_id = match resolve_pr_id(project_path, &req.pr_id).await {
            Ok(id) => id,
            Err(e) => {
                return Ok(Response::new(RestorePrResponse {
                    success: false,
                    error: e,
                    ..Default::default()
                }))
            }
        };

        match restore_pr(project_path, &pr_id).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Pr,
                    HookOperation::Restore,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(RestorePrResponse {
                    success: true,
                    error: String::new(),
                    pr: Some(pr_to_proto(&result.pr, priority_levels)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Pr,
                    HookOperation::Restore,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(RestorePrResponse {
                    success: false,
                    error: e.to_string(),
                    pr: None,
                    manifest: None,
                }))
            }
        }
    }

    async fn get_next_pr_number(
        &self,
        request: Request<GetNextPrNumberRequest>,
    ) -> Result<Response<GetNextPrNumberResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);
        let prs_path = get_centy_path(project_path).join("prs");

        match crate::item::entities::pr::reconcile::get_next_pr_display_number(&prs_path).await {
            Ok(next_number) => Ok(Response::new(GetNextPrNumberResponse { next_number })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // ============ Link RPCs ============

    async fn create_link(
        &self,
        request: Request<CreateLinkRequest>,
    ) -> Result<Response<CreateLinkResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.source_id.clone();
        let hook_request_data = serde_json::json!({
            "source_id": &req.source_id,
            "target_id": &req.target_id,
            "link_type": &req.link_type,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Link,
            HookOperation::Create,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(CreateLinkResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        // Convert proto types to internal types
        let source_type = proto_link_target_to_internal(req.source_type());
        let target_type = proto_link_target_to_internal(req.target_type());

        // Get custom link types from config
        let custom_types = match read_config(project_path).await {
            Ok(Some(config)) => config.custom_link_types,
            Ok(None) => vec![],
            Err(_) => vec![],
        };

        let options = CreateLinkOptions {
            source_id: req.source_id,
            source_type,
            target_id: req.target_id,
            target_type,
            link_type: req.link_type,
        };

        match create_link(project_path, options, &custom_types).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Link,
                    HookOperation::Create,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(CreateLinkResponse {
                    success: true,
                    error: String::new(),
                    created_link: Some(internal_link_to_proto(&result.created_link)),
                    inverse_link: Some(internal_link_to_proto(&result.inverse_link)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Link,
                    HookOperation::Create,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(CreateLinkResponse {
                    success: false,
                    error: e.to_string(),
                    created_link: None,
                    inverse_link: None,
                }))
            }
        }
    }

    async fn delete_link(
        &self,
        request: Request<DeleteLinkRequest>,
    ) -> Result<Response<DeleteLinkResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.source_id.clone();
        let hook_request_data = serde_json::json!({
            "source_id": &req.source_id,
            "target_id": &req.target_id,
            "link_type": &req.link_type,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::Link,
            HookOperation::Delete,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(DeleteLinkResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        // Convert proto types to internal types
        let source_type = proto_link_target_to_internal(req.source_type());
        let target_type = proto_link_target_to_internal(req.target_type());

        // Get custom link types from config
        let custom_types = match read_config(project_path).await {
            Ok(Some(config)) => config.custom_link_types,
            Ok(None) => vec![],
            Err(_) => vec![],
        };

        let options = DeleteLinkOptions {
            source_id: req.source_id,
            source_type,
            target_id: req.target_id,
            target_type,
            link_type: if req.link_type.is_empty() {
                None
            } else {
                Some(req.link_type)
            },
        };

        match delete_link(project_path, options, &custom_types).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Link,
                    HookOperation::Delete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(DeleteLinkResponse {
                    success: true,
                    error: String::new(),
                    deleted_count: result.deleted_count,
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::Link,
                    HookOperation::Delete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(DeleteLinkResponse {
                    success: false,
                    error: e.to_string(),
                    deleted_count: 0,
                }))
            }
        }
    }

    async fn list_links(
        &self,
        request: Request<ListLinksRequest>,
    ) -> Result<Response<ListLinksResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Convert proto type to internal type
        let entity_type = proto_link_target_to_internal(req.entity_type());

        match list_links(project_path, &req.entity_id, entity_type).await {
            Ok(links_file) => Ok(Response::new(ListLinksResponse {
                links: links_file
                    .links
                    .iter()
                    .map(internal_link_to_proto)
                    .collect(),
                total_count: links_file.links.len() as i32,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_available_link_types(
        &self,
        request: Request<GetAvailableLinkTypesRequest>,
    ) -> Result<Response<GetAvailableLinkTypesResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Get custom link types from config
        let custom_types = match read_config(project_path).await {
            Ok(Some(config)) => config.custom_link_types,
            Ok(None) => vec![],
            Err(_) => vec![],
        };

        let types = get_available_link_types(&custom_types);

        Ok(Response::new(GetAvailableLinkTypesResponse {
            link_types: types
                .iter()
                .map(|t| LinkTypeInfo {
                    name: t.name.clone(),
                    inverse: t.inverse.clone(),
                    description: t.description.clone().unwrap_or_default(),
                    is_builtin: t.is_builtin,
                })
                .collect(),
        }))
    }

    // ============ User RPCs ============

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_request_data = serde_json::json!({
            "id": &req.id,
            "name": &req.name,
            "email": &req.email,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::User,
            HookOperation::Create,
            &hook_project_path,
            None,
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(CreateUserResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        let options = CreateUserOptions {
            id: req.id,
            name: req.name,
            email: if req.email.is_empty() {
                None
            } else {
                Some(req.email)
            },
            git_usernames: req.git_usernames,
        };

        match internal_create_user(project_path, options).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::User,
                    HookOperation::Create,
                    &hook_project_path,
                    Some(&result.user.id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(CreateUserResponse {
                    success: true,
                    error: String::new(),
                    user: Some(user_to_proto(&result.user)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::User,
                    HookOperation::Create,
                    &hook_project_path,
                    None,
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(CreateUserResponse {
                    success: false,
                    error: e.to_string(),
                    user: None,
                    manifest: None,
                }))
            }
        }
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match internal_get_user(project_path, &req.user_id).await {
            Ok(user) => Ok(Response::new(GetUserResponse {
                success: true,
                error: String::new(),
                user: Some(user_to_proto(&user)),
            })),
            Err(e) => Ok(Response::new(GetUserResponse {
                success: false,
                error: e.to_string(),
                user: None,
            })),
        }
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let filter = if req.git_username.is_empty() {
            None
        } else {
            Some(req.git_username.as_str())
        };

        match internal_list_users(project_path, filter, false).await {
            Ok(users) => {
                let total_count = users.len() as i32;
                Ok(Response::new(ListUsersResponse {
                    users: users.iter().map(user_to_proto).collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.user_id.clone();
        let hook_request_data = serde_json::json!({
            "user_id": &req.user_id,
            "name": &req.name,
            "email": &req.email,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::User,
            HookOperation::Update,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(UpdateUserResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        let options = UpdateUserOptions {
            name: if req.name.is_empty() {
                None
            } else {
                Some(req.name)
            },
            email: if req.email.is_empty() {
                None
            } else {
                Some(req.email)
            },
            git_usernames: if req.git_usernames.is_empty() {
                None
            } else {
                Some(req.git_usernames)
            },
        };

        match internal_update_user(project_path, &req.user_id, options).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::User,
                    HookOperation::Update,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(UpdateUserResponse {
                    success: true,
                    error: String::new(),
                    user: Some(user_to_proto(&result.user)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::User,
                    HookOperation::Update,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(UpdateUserResponse {
                    success: false,
                    error: e.to_string(),
                    user: None,
                    manifest: None,
                }))
            }
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.user_id.clone();
        let hook_request_data = serde_json::json!({
            "user_id": &req.user_id,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::User,
            HookOperation::Delete,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(DeleteUserResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        match internal_delete_user(project_path, &req.user_id).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::User,
                    HookOperation::Delete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(DeleteUserResponse {
                    success: true,
                    error: String::new(),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::User,
                    HookOperation::Delete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(DeleteUserResponse {
                    success: false,
                    error: e.to_string(),
                    manifest: None,
                }))
            }
        }
    }

    async fn soft_delete_user(
        &self,
        request: Request<SoftDeleteUserRequest>,
    ) -> Result<Response<SoftDeleteUserResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.user_id.clone();
        let hook_request_data = serde_json::json!({
            "user_id": &req.user_id,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::User,
            HookOperation::SoftDelete,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(SoftDeleteUserResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        match internal_soft_delete_user(project_path, &req.user_id).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::User,
                    HookOperation::SoftDelete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(SoftDeleteUserResponse {
                    success: true,
                    error: String::new(),
                    user: Some(user_to_proto(&result.user)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::User,
                    HookOperation::SoftDelete,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(SoftDeleteUserResponse {
                    success: false,
                    error: e.to_string(),
                    user: None,
                    manifest: None,
                }))
            }
        }
    }

    async fn restore_user(
        &self,
        request: Request<RestoreUserRequest>,
    ) -> Result<Response<RestoreUserResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Pre-hook
        let hook_project_path = req.project_path.clone();
        let hook_item_id = req.user_id.clone();
        let hook_request_data = serde_json::json!({
            "user_id": &req.user_id,
        });
        if let Err(e) = maybe_run_pre_hooks(
            project_path,
            HookItemType::User,
            HookOperation::Restore,
            &hook_project_path,
            Some(&hook_item_id),
            Some(hook_request_data.clone()),
        )
        .await
        {
            return Ok(Response::new(RestoreUserResponse {
                success: false,
                error: e,
                ..Default::default()
            }));
        }

        match internal_restore_user(project_path, &req.user_id).await {
            Ok(result) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::User,
                    HookOperation::Restore,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    true,
                )
                .await;

                Ok(Response::new(RestoreUserResponse {
                    success: true,
                    error: String::new(),
                    user: Some(user_to_proto(&result.user)),
                    manifest: Some(manifest_to_proto(&result.manifest)),
                }))
            }
            Err(e) => {
                maybe_run_post_hooks(
                    project_path,
                    HookItemType::User,
                    HookOperation::Restore,
                    &hook_project_path,
                    Some(&hook_item_id),
                    Some(hook_request_data),
                    false,
                )
                .await;

                Ok(Response::new(RestoreUserResponse {
                    success: false,
                    error: e.to_string(),
                    user: None,
                    manifest: None,
                }))
            }
        }
    }

    async fn sync_users(
        &self,
        request: Request<SyncUsersRequest>,
    ) -> Result<Response<SyncUsersResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match internal_sync_users(project_path, req.dry_run).await {
            Ok(full_result) => {
                let result = full_result.result;
                Ok(Response::new(SyncUsersResponse {
                    success: true,
                    error: String::new(),
                    created: result.created,
                    skipped: result.skipped,
                    errors: result.errors,
                    would_create: result
                        .would_create
                        .into_iter()
                        .map(|c| ProtoGitContributor {
                            name: c.name,
                            email: c.email,
                        })
                        .collect(),
                    would_skip: result
                        .would_skip
                        .into_iter()
                        .map(|c| ProtoGitContributor {
                            name: c.name,
                            email: c.email,
                        })
                        .collect(),
                    manifest: Some(manifest_to_proto(&full_result.manifest)),
                }))
            }
            Err(e) => Ok(Response::new(SyncUsersResponse {
                success: false,
                error: e.to_string(),
                created: vec![],
                skipped: vec![],
                errors: vec![],
                would_create: vec![],
                would_skip: vec![],
                manifest: None,
            })),
        }
    }

    async fn advanced_search(
        &self,
        request: Request<AdvancedSearchRequest>,
    ) -> Result<Response<AdvancedSearchResponse>, Status> {
        let req = request.into_inner();

        // Track project if single-project search
        if !req.multi_project && !req.project_path.is_empty() {
            track_project_async(req.project_path.clone());
        }

        // Parse sort options
        let sort = if req.sort_by.is_empty() {
            None
        } else {
            Some(SortOptions {
                field: req.sort_by.parse().unwrap(),
                descending: req.sort_descending,
            })
        };

        let options = SearchOptions {
            query: req.query,
            sort,
            multi_project: req.multi_project,
            project_path: if req.project_path.is_empty() {
                None
            } else {
                Some(req.project_path)
            },
        };

        match advanced_search(options).await {
            Ok(result) => {
                // Convert results to proto types
                let results: Vec<ProtoSearchResultIssue> = result
                    .results
                    .into_iter()
                    .map(|r| {
                        // Use default priority_levels since we can't do async in map
                        let priority_levels = 3;
                        ProtoSearchResultIssue {
                            issue: Some(issue_to_proto(&r.issue, priority_levels)),
                            project_path: r.project_path,
                            project_name: r.project_name,
                            display_path: r.display_path,
                        }
                    })
                    .collect();

                let total_count = results.len() as i32;

                Ok(Response::new(AdvancedSearchResponse {
                    success: true,
                    error: String::new(),
                    results,
                    total_count,
                    parsed_query: result.parsed_query,
                }))
            }
            Err(e) => Ok(Response::new(AdvancedSearchResponse {
                success: false,
                error: e.to_string(),
                results: vec![],
                total_count: 0,
                parsed_query: String::new(),
            })),
        }
    }

    async fn get_supported_editors(
        &self,
        _request: Request<GetSupportedEditorsRequest>,
    ) -> Result<Response<GetSupportedEditorsResponse>, Status> {
        let all_editors = get_all_editors().await;
        let editors: Vec<EditorInfo> = all_editors
            .iter()
            .map(|e| {
                let editor_type = match e.id.as_str() {
                    "vscode" => ProtoEditorType::Vscode,
                    "terminal" => ProtoEditorType::Terminal,
                    _ => ProtoEditorType::Unspecified,
                };
                EditorInfo {
                    editor_type: editor_type.into(),
                    name: e.name.clone(),
                    description: e.description.clone(),
                    available: is_editor_available(e),
                    editor_id: e.id.clone(),
                    terminal_wrapper: e.terminal_wrapper,
                }
            })
            .collect();

        Ok(Response::new(GetSupportedEditorsResponse { editors }))
    }

    async fn open_in_temp_workspace(
        &self,
        request: Request<OpenInTempWorkspaceWithEditorRequest>,
    ) -> Result<Response<OpenInTempWorkspaceResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let err_response = |error: String, issue_id: String, dn: u32, req_cfg: bool| {
            Ok(Response::new(OpenInTempWorkspaceResponse {
                success: false,
                error,
                workspace_path: String::new(),
                issue_id,
                display_number: dn,
                expires_at: String::new(),
                editor_opened: false,
                requires_status_config: req_cfg,
                workspace_reused: false,
                original_created_at: String::new(),
            }))
        };

        let action_str = match req.action {
            1 => "plan",
            2 => "implement",
            _ => "plan",
        };

        let issue = match resolve_issue(project_path, &req.issue_id).await {
            Ok(i) => i,
            Err(e) => return err_response(e, String::new(), 0, false),
        };

        let config = read_config(project_path).await.ok().flatten();
        let requires_status_config = config
            .as_ref()
            .map(|c| c.llm.update_status_on_start.is_none())
            .unwrap_or(true);
        if requires_status_config {
            return err_response(
                "Status update preference not configured. Run 'centy config --update-status-on-start true' to enable automatic status updates.".to_string(),
                issue.id.clone(), issue.metadata.display_number, true,
            );
        }

        if let Some(ref cfg) = config {
            if cfg.llm.update_status_on_start == Some(true)
                && issue.metadata.status != "in-progress"
                && issue.metadata.status != "closed"
            {
                let _ = update_issue(
                    project_path,
                    &issue.id,
                    UpdateIssueOptions {
                        status: Some("in-progress".to_string()),
                        ..Default::default()
                    },
                )
                .await;
            }
        }

        let agent_name = if req.agent_name.is_empty() {
            "claude".to_string()
        } else {
            req.agent_name.clone()
        };

        // Resolve editor ID from request, project config, or user defaults
        let project_default = config.as_ref().and_then(|c| c.default_editor.as_deref());
        let editor_id = resolve_editor_id(Some(&req.editor_id), project_default).await;

        match create_temp_workspace(CreateWorkspaceOptions {
            source_project_path: project_path.to_path_buf(),
            issue: issue.clone(),
            action: action_str.to_string(),
            agent_name,
            ttl_hours: req.ttl_hours,
            editor: EditorType::from_id(&editor_id),
        })
        .await
        {
            Ok(result) => Ok(Response::new(OpenInTempWorkspaceResponse {
                success: true,
                error: String::new(),
                workspace_path: result.workspace_path.to_string_lossy().to_string(),
                issue_id: issue.id.clone(),
                display_number: issue.metadata.display_number,
                expires_at: result.entry.expires_at,
                editor_opened: result.editor_opened,
                requires_status_config: false,
                workspace_reused: result.workspace_reused,
                original_created_at: result.original_created_at.unwrap_or_default(),
            })),
            Err(e) => err_response(e.to_string(), String::new(), 0, false),
        }
    }

    // Deprecated: thin wrapper delegating to unified OpenInTempWorkspace with "vscode" editor
    async fn open_in_temp_vscode(
        &self,
        request: Request<OpenInTempWorkspaceRequest>,
    ) -> Result<Response<OpenInTempWorkspaceResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let err_response = |error: String, issue_id: String, dn: u32, req_cfg: bool| {
            Ok(Response::new(OpenInTempWorkspaceResponse {
                success: false,
                error,
                workspace_path: String::new(),
                issue_id,
                display_number: dn,
                expires_at: String::new(),
                editor_opened: false,
                requires_status_config: req_cfg,
                workspace_reused: false,
                original_created_at: String::new(),
            }))
        };

        let action_str = match req.action {
            1 => "plan",
            2 => "implement",
            _ => "plan",
        };

        let issue = match resolve_issue(project_path, &req.issue_id).await {
            Ok(i) => i,
            Err(e) => return err_response(e, String::new(), 0, false),
        };

        let config = read_config(project_path).await.ok().flatten();
        let requires_status_config = config
            .as_ref()
            .map(|c| c.llm.update_status_on_start.is_none())
            .unwrap_or(true);
        if requires_status_config {
            return err_response(
                "Status update preference not configured. Run 'centy config --update-status-on-start true' to enable automatic status updates.".to_string(),
                issue.id.clone(), issue.metadata.display_number, true,
            );
        }

        if let Some(ref cfg) = config {
            if cfg.llm.update_status_on_start == Some(true)
                && issue.metadata.status != "in-progress"
                && issue.metadata.status != "closed"
            {
                let _ = update_issue(
                    project_path,
                    &issue.id,
                    UpdateIssueOptions {
                        status: Some("in-progress".to_string()),
                        ..Default::default()
                    },
                )
                .await;
            }
        }

        let agent_name = if req.agent_name.is_empty() {
            "claude".to_string()
        } else {
            req.agent_name.clone()
        };

        match create_temp_workspace(CreateWorkspaceOptions {
            source_project_path: project_path.to_path_buf(),
            issue: issue.clone(),
            action: action_str.to_string(),
            agent_name,
            ttl_hours: req.ttl_hours,
            editor: EditorType::VSCode,
        })
        .await
        {
            Ok(result) => Ok(Response::new(OpenInTempWorkspaceResponse {
                success: true,
                error: String::new(),
                workspace_path: result.workspace_path.to_string_lossy().to_string(),
                issue_id: issue.id.clone(),
                display_number: issue.metadata.display_number,
                expires_at: result.entry.expires_at,
                editor_opened: result.editor_opened,
                requires_status_config: false,
                workspace_reused: result.workspace_reused,
                original_created_at: result.original_created_at.unwrap_or_default(),
            })),
            Err(e) => err_response(e.to_string(), String::new(), 0, false),
        }
    }

    async fn open_in_temp_terminal(
        &self,
        request: Request<OpenInTempWorkspaceRequest>,
    ) -> Result<Response<OpenInTempWorkspaceResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let action_str = match req.action {
            1 => "plan",
            2 => "implement",
            _ => "plan",
        };

        let err_response = |error: String, issue_id: String, dn: u32, req_cfg: bool| {
            Ok(Response::new(OpenInTempWorkspaceResponse {
                success: false,
                error,
                workspace_path: String::new(),
                issue_id,
                display_number: dn,
                expires_at: String::new(),
                editor_opened: false,
                requires_status_config: req_cfg,
                workspace_reused: false,
                original_created_at: String::new(),
            }))
        };

        let issue = match resolve_issue(project_path, &req.issue_id).await {
            Ok(i) => i,
            Err(e) => return err_response(e, String::new(), 0, false),
        };

        let config = read_config(project_path).await.ok().flatten();
        if config
            .as_ref()
            .map(|c| c.llm.update_status_on_start.is_none())
            .unwrap_or(true)
        {
            return err_response(
                "Status update preference not configured".to_string(),
                issue.id.clone(),
                issue.metadata.display_number,
                true,
            );
        }

        if let Some(ref cfg) = config {
            if cfg.llm.update_status_on_start == Some(true) {
                let current_status = &issue.metadata.status;
                if current_status != "in-progress" && current_status != "closed" {
                    let _ = update_issue(
                        project_path,
                        &issue.id,
                        UpdateIssueOptions {
                            status: Some("in-progress".to_string()),
                            ..Default::default()
                        },
                    )
                    .await;
                }
            }
        }

        let agent_name = if req.agent_name.is_empty() {
            "claude".to_string()
        } else {
            req.agent_name.clone()
        };

        match create_temp_workspace(CreateWorkspaceOptions {
            source_project_path: project_path.to_path_buf(),
            issue: issue.clone(),
            action: action_str.to_string(),
            agent_name,
            ttl_hours: req.ttl_hours,
            editor: EditorType::Terminal,
        })
        .await
        {
            Ok(result) => Ok(Response::new(OpenInTempWorkspaceResponse {
                success: true,
                error: String::new(),
                workspace_path: result.workspace_path.to_string_lossy().to_string(),
                issue_id: issue.id.clone(),
                display_number: issue.metadata.display_number,
                expires_at: result.entry.expires_at,
                editor_opened: result.editor_opened,
                requires_status_config: false,
                workspace_reused: result.workspace_reused,
                original_created_at: result.original_created_at.unwrap_or_default(),
            })),
            Err(e) => err_response(e.to_string(), String::new(), 0, false),
        }
    }

    async fn open_agent_in_terminal(
        &self,
        request: Request<OpenAgentInTerminalRequest>,
    ) -> Result<Response<OpenAgentInTerminalResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let err_response = |error: String, issue_id: String, dn: u32, req_cfg: bool| {
            Ok(Response::new(OpenAgentInTerminalResponse {
                success: false,
                error,
                working_directory: String::new(),
                issue_id,
                display_number: dn,
                agent_command: String::new(),
                terminal_opened: false,
                expires_at: String::new(),
                requires_status_config: req_cfg,
            }))
        };

        let issue = match resolve_issue(project_path, &req.issue_id).await {
            Ok(i) => i,
            Err(e) => return err_response(e, String::new(), 0, false),
        };

        let config = read_config(project_path).await.ok().flatten();
        let requires_status_config = config
            .as_ref()
            .map(|c| c.llm.update_status_on_start.is_none())
            .unwrap_or(true);
        if requires_status_config {
            return err_response(
                String::new(),
                issue.id.clone(),
                issue.metadata.display_number,
                true,
            );
        }

        if let Some(ref cfg) = config {
            if cfg.llm.update_status_on_start == Some(true)
                && issue.metadata.status != "in-progress"
                && issue.metadata.status != "closed"
            {
                let _ = update_issue(
                    project_path,
                    &issue.id,
                    UpdateIssueOptions {
                        status: Some("in-progress".to_string()),
                        ..Default::default()
                    },
                )
                .await;
            }
        }

        let agent_name = if req.agent_name.is_empty() {
            "claude".to_string()
        } else {
            req.agent_name.clone()
        };

        // Use agent name as command (e.g., "claude" -> "claude" command)
        let agent_command = agent_name.clone();
        let agent_args: Vec<String> = Vec::new();

        let workspace_mode = match req.workspace_mode {
            x if x == WorkspaceMode::Temp as i32 => WorkspaceMode::Temp,
            x if x == WorkspaceMode::Current as i32 => WorkspaceMode::Current,
            _ => config
                .as_ref()
                .map(|c| match c.llm.default_workspace_mode {
                    x if x == WorkspaceMode::Temp as i32 => WorkspaceMode::Temp,
                    _ => WorkspaceMode::Current,
                })
                .unwrap_or(WorkspaceMode::Current),
        };

        let (working_dir, expires_at) = match workspace_mode {
            WorkspaceMode::Temp => match create_temp_workspace(CreateWorkspaceOptions {
                source_project_path: project_path.to_path_buf(),
                issue: issue.clone(),
                action: "agent".to_string(),
                agent_name: agent_name.clone(),
                ttl_hours: req.ttl_hours,
                editor: EditorType::None, // Terminal is opened separately via open_terminal_with_agent
            })
            .await
            {
                Ok(r) => (r.workspace_path, Some(r.entry.expires_at)),
                Err(e) => {
                    return err_response(
                        format!("Failed to create workspace: {e}"),
                        String::new(),
                        0,
                        false,
                    )
                }
            },
            _ => (project_path.to_path_buf(), None),
        };

        let terminal_opened = open_terminal_with_agent(
            &working_dir,
            issue.metadata.display_number,
            &agent_command,
            &agent_args,
            None,
        )
        .unwrap_or(false);
        let full_command = agent_command.clone();

        Ok(Response::new(OpenAgentInTerminalResponse {
            success: true,
            error: String::new(),
            working_directory: working_dir.to_string_lossy().to_string(),
            issue_id: issue.id.clone(),
            display_number: issue.metadata.display_number,
            agent_command: full_command,
            terminal_opened,
            expires_at: expires_at.unwrap_or_default(),
            requires_status_config: false,
        }))
    }

    async fn list_temp_workspaces(
        &self,
        request: Request<ListTempWorkspacesRequest>,
    ) -> Result<Response<ListTempWorkspacesResponse>, Status> {
        let req = request.into_inner();

        let source_filter = if req.source_project_path.is_empty() {
            None
        } else {
            Some(req.source_project_path.as_str())
        };

        match internal_list_workspaces(req.include_expired, source_filter).await {
            Ok(workspaces) => {
                let expired_count = workspaces.iter().filter(|(_, _, exp)| *exp).count() as u32;
                let proto_workspaces: Vec<ProtoTempWorkspace> = workspaces
                    .into_iter()
                    .map(|(path, entry, _)| ProtoTempWorkspace {
                        workspace_path: path,
                        source_project_path: entry.source_project_path,
                        issue_id: entry.issue_id,
                        issue_display_number: entry.issue_display_number,
                        issue_title: entry.issue_title,
                        agent_name: entry.agent_name,
                        action: match entry.action.as_str() {
                            "plan" => 1,       // LLM_ACTION_PLAN
                            "implement" => 2,  // LLM_ACTION_IMPLEMENT
                            "standalone" => 0, // No specific action for standalone
                            _ => 0,
                        },
                        created_at: entry.created_at,
                        expires_at: entry.expires_at,
                        is_standalone: entry.is_standalone,
                        workspace_id: entry.workspace_id,
                        workspace_name: entry.workspace_name,
                        workspace_description: entry.workspace_description,
                    })
                    .collect();

                Ok(Response::new(ListTempWorkspacesResponse {
                    total_count: proto_workspaces.len() as u32,
                    workspaces: proto_workspaces,
                    expired_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn close_temp_workspace(
        &self,
        request: Request<CloseTempWorkspaceRequest>,
    ) -> Result<Response<CloseTempWorkspaceResponse>, Status> {
        let req = request.into_inner();

        match internal_cleanup_workspace(&req.workspace_path, req.force).await {
            Ok(result) => Ok(Response::new(CloseTempWorkspaceResponse {
                success: result.error.is_none(),
                error: result.error.unwrap_or_default(),
                worktree_removed: result.worktree_removed,
                directory_removed: result.directory_removed,
            })),
            Err(e) => Ok(Response::new(CloseTempWorkspaceResponse {
                success: false,
                error: e.to_string(),
                worktree_removed: false,
                directory_removed: false,
            })),
        }
    }

    async fn cleanup_expired_workspaces(
        &self,
        _request: Request<CleanupExpiredWorkspacesRequest>,
    ) -> Result<Response<CleanupExpiredWorkspacesResponse>, Status> {
        match internal_cleanup_expired().await {
            Ok(results) => {
                let cleaned_count = results.iter().filter(|r| r.error.is_none()).count() as u32;
                let cleaned_paths: Vec<String> = results
                    .iter()
                    .filter(|r| r.error.is_none())
                    .map(|r| r.workspace_path.clone())
                    .collect();
                let failed_paths: Vec<String> = results
                    .iter()
                    .filter(|r| r.error.is_some())
                    .map(|r| r.workspace_path.clone())
                    .collect();

                Ok(Response::new(CleanupExpiredWorkspacesResponse {
                    success: true,
                    error: String::new(),
                    cleaned_count,
                    cleaned_paths,
                    failed_paths,
                }))
            }
            Err(e) => Ok(Response::new(CleanupExpiredWorkspacesResponse {
                success: false,
                error: e.to_string(),
                cleaned_count: 0,
                cleaned_paths: vec![],
                failed_paths: vec![],
            })),
        }
    }

    async fn open_standalone_workspace(
        &self,
        request: Request<OpenStandaloneWorkspaceWithEditorRequest>,
    ) -> Result<Response<OpenStandaloneWorkspaceResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let err_response = |error: String| {
            Ok(Response::new(OpenStandaloneWorkspaceResponse {
                success: false,
                error,
                workspace_path: String::new(),
                workspace_id: String::new(),
                name: String::new(),
                expires_at: String::new(),
                editor_opened: false,
                workspace_reused: false,
                original_created_at: String::new(),
            }))
        };

        let agent_name = if req.agent_name.is_empty() {
            "claude".to_string()
        } else {
            req.agent_name.clone()
        };

        let name = if req.name.is_empty() {
            None
        } else {
            Some(req.name.clone())
        };

        let description = if req.description.is_empty() {
            None
        } else {
            Some(req.description.clone())
        };

        // Resolve editor ID from request, project config, or user defaults
        let config = read_config(project_path).await.ok().flatten();
        let project_default = config.as_ref().and_then(|c| c.default_editor.as_deref());
        let editor_id = resolve_editor_id(Some(&req.editor_id), project_default).await;

        match create_standalone_workspace(CreateStandaloneWorkspaceOptions {
            source_project_path: project_path.to_path_buf(),
            name,
            description,
            ttl_hours: req.ttl_hours,
            agent_name,
            editor: EditorType::from_id(&editor_id),
        })
        .await
        {
            Ok(result) => Ok(Response::new(OpenStandaloneWorkspaceResponse {
                success: true,
                error: String::new(),
                workspace_path: result.workspace_path.to_string_lossy().to_string(),
                workspace_id: result.entry.workspace_id.clone(),
                name: result.entry.workspace_name.clone(),
                expires_at: result.entry.expires_at,
                editor_opened: result.editor_opened,
                workspace_reused: result.workspace_reused,
                original_created_at: result.original_created_at.unwrap_or_default(),
            })),
            Err(e) => err_response(e.to_string()),
        }
    }

    // Deprecated: thin wrapper delegating to unified OpenStandaloneWorkspace with "vscode" editor
    async fn open_standalone_workspace_vscode(
        &self,
        request: Request<OpenStandaloneWorkspaceRequest>,
    ) -> Result<Response<OpenStandaloneWorkspaceResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let err_response = |error: String| {
            Ok(Response::new(OpenStandaloneWorkspaceResponse {
                success: false,
                error,
                workspace_path: String::new(),
                workspace_id: String::new(),
                name: String::new(),
                expires_at: String::new(),
                editor_opened: false,
                workspace_reused: false,
                original_created_at: String::new(),
            }))
        };

        let agent_name = if req.agent_name.is_empty() {
            "claude".to_string()
        } else {
            req.agent_name.clone()
        };

        let name = if req.name.is_empty() {
            None
        } else {
            Some(req.name.clone())
        };

        let description = if req.description.is_empty() {
            None
        } else {
            Some(req.description.clone())
        };

        match create_standalone_workspace(CreateStandaloneWorkspaceOptions {
            source_project_path: project_path.to_path_buf(),
            name,
            description,
            ttl_hours: req.ttl_hours,
            agent_name,
            editor: EditorType::VSCode,
        })
        .await
        {
            Ok(result) => Ok(Response::new(OpenStandaloneWorkspaceResponse {
                success: true,
                error: String::new(),
                workspace_path: result.workspace_path.to_string_lossy().to_string(),
                workspace_id: result.entry.workspace_id.clone(),
                name: result.entry.workspace_name.clone(),
                expires_at: result.entry.expires_at,
                editor_opened: result.editor_opened,
                workspace_reused: result.workspace_reused,
                original_created_at: result.original_created_at.unwrap_or_default(),
            })),
            Err(e) => err_response(e.to_string()),
        }
    }

    async fn open_standalone_workspace_terminal(
        &self,
        request: Request<OpenStandaloneWorkspaceRequest>,
    ) -> Result<Response<OpenStandaloneWorkspaceResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let err_response = |error: String| {
            Ok(Response::new(OpenStandaloneWorkspaceResponse {
                success: false,
                error,
                workspace_path: String::new(),
                workspace_id: String::new(),
                name: String::new(),
                expires_at: String::new(),
                editor_opened: false,
                workspace_reused: false,
                original_created_at: String::new(),
            }))
        };

        let agent_name = if req.agent_name.is_empty() {
            "claude".to_string()
        } else {
            req.agent_name.clone()
        };

        let name = if req.name.is_empty() {
            None
        } else {
            Some(req.name.clone())
        };

        let description = if req.description.is_empty() {
            None
        } else {
            Some(req.description.clone())
        };

        match create_standalone_workspace(CreateStandaloneWorkspaceOptions {
            source_project_path: project_path.to_path_buf(),
            name,
            description,
            ttl_hours: req.ttl_hours,
            agent_name,
            editor: EditorType::Terminal,
        })
        .await
        {
            Ok(result) => Ok(Response::new(OpenStandaloneWorkspaceResponse {
                success: true,
                error: String::new(),
                workspace_path: result.workspace_path.to_string_lossy().to_string(),
                workspace_id: result.entry.workspace_id.clone(),
                name: result.entry.workspace_name.clone(),
                expires_at: result.entry.expires_at,
                editor_opened: result.editor_opened,
                workspace_reused: result.workspace_reused,
                original_created_at: result.original_created_at.unwrap_or_default(),
            })),
            Err(e) => err_response(e.to_string()),
        }
    }

    async fn get_entity_actions(
        &self,
        request: Request<GetEntityActionsRequest>,
    ) -> Result<Response<GetEntityActionsResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let config = read_config(project_path).await.ok().flatten();
        let allowed_states = config
            .as_ref()
            .map(|c| c.allowed_states.clone())
            .unwrap_or_else(|| {
                vec![
                    "open".to_string(),
                    "in-progress".to_string(),
                    "closed".to_string(),
                ]
            });

        let has_entity_id = !req.entity_id.is_empty();

        let actions = match req.entity_type {
            t if t == EntityType::Issue as i32 => {
                let entity_status = if has_entity_id {
                    resolve_issue(project_path, &req.entity_id)
                        .await
                        .ok()
                        .map(|i| i.metadata.status)
                } else {
                    None
                };
                build_issue_actions(
                    entity_status.as_ref(),
                    &allowed_states,
                    is_vscode_available(),
                    is_terminal_available(),
                    has_entity_id,
                )
            }
            t if t == EntityType::Pr as i32 => {
                let entity_status = if has_entity_id {
                    if let Ok(n) = req.entity_id.parse::<u32>() {
                        get_pr_by_display_number(project_path, n)
                            .await
                            .ok()
                            .map(|p| p.metadata.status)
                    } else {
                        get_pr(project_path, &req.entity_id)
                            .await
                            .ok()
                            .map(|p| p.metadata.status)
                    }
                } else {
                    None
                };
                build_pr_actions(entity_status.as_ref(), has_entity_id)
            }
            t if t == EntityType::Doc as i32 => build_doc_actions(has_entity_id),
            _ => {
                return Ok(Response::new(GetEntityActionsResponse {
                    actions: vec![],
                    success: false,
                    error: "Unknown entity type".to_string(),
                }))
            }
        };

        Ok(Response::new(GetEntityActionsResponse {
            actions,
            success: true,
            error: String::new(),
        }))
    }

    // ============ Sync RPCs (Stubbed - sync feature removed) ============

    async fn list_sync_conflicts(
        &self,
        _request: Request<proto::ListSyncConflictsRequest>,
    ) -> Result<Response<proto::ListSyncConflictsResponse>, Status> {
        // Sync feature removed - return empty list
        Ok(Response::new(proto::ListSyncConflictsResponse {
            conflicts: vec![],
            success: true,
            error: String::new(),
        }))
    }

    async fn get_sync_conflict(
        &self,
        request: Request<proto::GetSyncConflictRequest>,
    ) -> Result<Response<proto::GetSyncConflictResponse>, Status> {
        let req = request.into_inner();
        // Sync feature removed - conflict not found
        Ok(Response::new(proto::GetSyncConflictResponse {
            conflict: None,
            success: false,
            error: format!(
                "Sync feature disabled. Conflict not found: {}",
                req.conflict_id
            ),
        }))
    }

    async fn resolve_sync_conflict(
        &self,
        _request: Request<proto::ResolveSyncConflictRequest>,
    ) -> Result<Response<proto::ResolveSyncConflictResponse>, Status> {
        // Sync feature removed - cannot resolve conflicts
        Ok(Response::new(proto::ResolveSyncConflictResponse {
            success: false,
            error: "Sync feature is disabled".to_string(),
        }))
    }

    async fn get_sync_status(
        &self,
        _request: Request<proto::GetSyncStatusRequest>,
    ) -> Result<Response<proto::GetSyncStatusResponse>, Status> {
        // Sync feature removed - return disabled status
        Ok(Response::new(proto::GetSyncStatusResponse {
            mode: proto::SyncMode::Disabled as i32,
            has_pending_changes: false,
            has_pending_push: false,
            conflict_count: 0,
            last_sync_time: String::new(),
            success: true,
            error: String::new(),
        }))
    }

    async fn sync_pull(
        &self,
        _request: Request<proto::SyncPullRequest>,
    ) -> Result<Response<proto::SyncPullResponse>, Status> {
        // Sync feature removed - no-op success
        Ok(Response::new(proto::SyncPullResponse {
            success: true,
            error: String::new(),
            had_changes: false,
            conflict_files: vec![],
        }))
    }

    async fn sync_push(
        &self,
        _request: Request<proto::SyncPushRequest>,
    ) -> Result<Response<proto::SyncPushResponse>, Status> {
        // Sync feature removed - no-op success
        Ok(Response::new(proto::SyncPushResponse {
            success: true,
            error: String::new(),
            had_changes: false,
        }))
    }
}

/// Helper to capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// Helper functions for link type conversion

fn proto_link_target_to_internal(proto_type: LinkTargetType) -> TargetType {
    match proto_type {
        LinkTargetType::Issue => TargetType::Issue,
        LinkTargetType::Doc => TargetType::Doc,
        LinkTargetType::Pr => TargetType::Pr,
        LinkTargetType::Unspecified => TargetType::Issue, // Default to issue
    }
}

fn internal_target_type_to_proto(internal_type: TargetType) -> i32 {
    match internal_type {
        TargetType::Issue => LinkTargetType::Issue as i32,
        TargetType::Doc => LinkTargetType::Doc as i32,
        TargetType::Pr => LinkTargetType::Pr as i32,
    }
}

fn internal_link_to_proto(link: &crate::link::Link) -> ProtoLink {
    ProtoLink {
        target_id: link.target_id.clone(),
        target_type: internal_target_type_to_proto(link.target_type),
        link_type: link.link_type.clone(),
        created_at: link.created_at.clone(),
    }
}

// Helper functions for converting internal types to proto types

fn manifest_to_proto(manifest: &InternalManifest) -> Manifest {
    Manifest {
        schema_version: manifest.schema_version as i32,
        centy_version: manifest.centy_version.clone(),
        created_at: manifest.created_at.clone(),
        updated_at: manifest.updated_at.clone(),
    }
}

fn file_info_to_proto(info: crate::reconciliation::FileInfo) -> FileInfo {
    FileInfo {
        path: info.path,
        file_type: match info.file_type {
            InternalFileType::File => FileType::File as i32,
            InternalFileType::Directory => FileType::Directory as i32,
        },
        hash: info.hash,
        content_preview: info.content_preview.unwrap_or_default(),
    }
}

fn config_to_proto(config: &CentyConfig) -> Config {
    Config {
        custom_fields: config
            .custom_fields
            .iter()
            .map(|f| CustomFieldDefinition {
                name: f.name.clone(),
                field_type: f.field_type.clone(),
                required: f.required,
                default_value: f.default_value.clone().unwrap_or_default(),
                enum_values: f.enum_values.clone(),
            })
            .collect(),
        defaults: config.defaults.clone(),
        priority_levels: config.priority_levels as i32,
        allowed_states: config.allowed_states.clone(),
        default_state: config.default_state.clone(),
        version: config.effective_version(),
        state_colors: config.state_colors.clone(),
        priority_colors: config.priority_colors.clone(),
        llm: Some(LlmConfig {
            auto_close_on_complete: config.llm.auto_close_on_complete,
            update_status_on_start: config.llm.update_status_on_start,
            allow_direct_edits: config.llm.allow_direct_edits,
            default_workspace_mode: config.llm.default_workspace_mode,
        }),
        custom_link_types: config
            .custom_link_types
            .iter()
            .map(|lt| LinkTypeDefinition {
                name: lt.name.clone(),
                inverse: lt.inverse.clone(),
                description: lt.description.clone().unwrap_or_default(),
            })
            .collect(),
        default_editor: config.default_editor.clone().unwrap_or_default(),
        hooks: config
            .hooks
            .iter()
            .map(|h| ProtoHookDefinition {
                pattern: h.pattern.clone(),
                command: h.command.clone(),
                run_async: h.is_async,
                timeout: h.timeout,
                enabled: h.enabled,
            })
            .collect(),
    }
}

fn proto_to_config(proto: &Config) -> CentyConfig {
    let llm_config = proto
        .llm
        .as_ref()
        .map(|l| InternalLlmConfig {
            auto_close_on_complete: l.auto_close_on_complete,
            update_status_on_start: l.update_status_on_start,
            allow_direct_edits: l.allow_direct_edits,
            default_workspace_mode: l.default_workspace_mode,
        })
        .unwrap_or_default();

    CentyConfig {
        version: if proto.version.is_empty() {
            None
        } else {
            Some(proto.version.clone())
        },
        priority_levels: proto.priority_levels as u32,
        custom_fields: proto
            .custom_fields
            .iter()
            .map(|f| InternalCustomFieldDef {
                name: f.name.clone(),
                field_type: f.field_type.clone(),
                required: f.required,
                default_value: if f.default_value.is_empty() {
                    None
                } else {
                    Some(f.default_value.clone())
                },
                enum_values: f.enum_values.clone(),
            })
            .collect(),
        defaults: proto.defaults.clone(),
        allowed_states: proto.allowed_states.clone(),
        default_state: proto.default_state.clone(),
        state_colors: proto.state_colors.clone(),
        priority_colors: proto.priority_colors.clone(),
        llm: llm_config,
        custom_link_types: proto
            .custom_link_types
            .iter()
            .map(|lt| crate::link::CustomLinkTypeDefinition {
                name: lt.name.clone(),
                inverse: lt.inverse.clone(),
                description: if lt.description.is_empty() {
                    None
                } else {
                    Some(lt.description.clone())
                },
            })
            .collect(),
        default_editor: if proto.default_editor.is_empty() {
            None
        } else {
            Some(proto.default_editor.clone())
        },
        hooks: proto
            .hooks
            .iter()
            .map(|h| InternalHookDefinition {
                pattern: h.pattern.clone(),
                command: h.command.clone(),
                is_async: h.run_async,
                timeout: if h.timeout == 0 { 30 } else { h.timeout },
                enabled: h.enabled,
            })
            .collect(),
    }
}

/// Validate the config and return an error message if invalid
fn validate_config(config: &CentyConfig) -> Result<(), String> {
    // Check allowed_states is not empty
    if config.allowed_states.is_empty() {
        return Err("allowed_states must not be empty".to_string());
    }

    // Check default_state is in allowed_states
    if !config.allowed_states.contains(&config.default_state) {
        return Err(format!(
            "default_state '{}' must be in allowed_states",
            config.default_state
        ));
    }

    // Check priority_levels is in valid range
    if config.priority_levels < 1 || config.priority_levels > 10 {
        return Err("priority_levels must be between 1 and 10".to_string());
    }

    // Check custom field names are unique
    let mut field_names = std::collections::HashSet::new();
    for field in &config.custom_fields {
        if !field_names.insert(&field.name) {
            return Err(format!("duplicate custom field name: '{}'", field.name));
        }

        // Check enum fields have values
        if field.field_type == "enum" && field.enum_values.is_empty() {
            return Err(format!(
                "custom field '{}' is of type 'enum' but has no enum_values",
                field.name
            ));
        }
    }

    // Validate color formats (hex colors)
    for (state, color) in &config.state_colors {
        if !HEX_COLOR_REGEX.is_match(color) {
            return Err(format!(
                "invalid color '{color}' for state '{state}': must be hex format (#RGB or #RRGGBB)"
            ));
        }
    }
    for (priority, color) in &config.priority_colors {
        if !HEX_COLOR_REGEX.is_match(color) {
            return Err(format!(
                "invalid color '{color}' for priority '{priority}': must be hex format (#RGB or #RRGGBB)"
            ));
        }
    }

    // Validate hooks
    for hook in &config.hooks {
        if hook.command.is_empty() {
            return Err(format!(
                "hook with pattern '{}' has an empty command",
                hook.pattern
            ));
        }
        if hook.timeout == 0 || hook.timeout > 300 {
            return Err(format!(
                "hook '{}' timeout must be between 1 and 300 seconds, got {}",
                hook.pattern, hook.timeout
            ));
        }
        // Validate pattern syntax
        if let Err(e) = crate::hooks::config::ParsedPattern::parse(&hook.pattern) {
            return Err(format!("invalid hook pattern '{}': {e}", hook.pattern));
        }
        // Async is only valid for post-hooks
        if hook.is_async {
            let parsed = crate::hooks::config::ParsedPattern::parse(&hook.pattern)
                .map_err(|e| format!("invalid hook pattern: {e}"))?;
            if let crate::hooks::config::PatternSegment::Exact(ref phase) = parsed.phase {
                if phase == "pre" {
                    return Err(format!(
                        "hook '{}' cannot be async: pre-hooks must be synchronous",
                        hook.pattern
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Build pre-hook context and run pre-hooks. Returns Err(String) if blocked.
async fn maybe_run_pre_hooks(
    project_path: &Path,
    item_type: HookItemType,
    operation: HookOperation,
    project_path_str: &str,
    item_id: Option<&str>,
    request_data: Option<serde_json::Value>,
) -> Result<(), String> {
    let context = HookContext::new(
        Phase::Pre,
        item_type,
        operation,
        project_path_str,
        item_id,
        request_data,
        None,
    );
    run_pre_hooks(project_path, item_type, operation, &context)
        .await
        .map_err(|e| format!("Hook blocked operation: {e}"))
}

/// Build post-hook context and run post-hooks (sync ones block, async ones are spawned).
async fn maybe_run_post_hooks(
    project_path: &Path,
    item_type: HookItemType,
    operation: HookOperation,
    project_path_str: &str,
    item_id: Option<&str>,
    request_data: Option<serde_json::Value>,
    success: bool,
) {
    let context = HookContext::new(
        Phase::Post,
        item_type,
        operation,
        project_path_str,
        item_id,
        request_data,
        Some(success),
    );
    run_post_hooks(project_path, item_type, operation, &context).await;
}

#[allow(deprecated)]
fn issue_to_proto(issue: &crate::item::entities::issue::Issue, priority_levels: u32) -> Issue {
    Issue {
        id: issue.id.clone(),
        display_number: issue.metadata.display_number,
        issue_number: issue.issue_number.clone(), // Legacy
        title: issue.title.clone(),
        description: issue.description.clone(),
        metadata: Some(IssueMetadata {
            display_number: issue.metadata.display_number,
            status: issue.metadata.status.clone(),
            priority: issue.metadata.priority as i32,
            created_at: issue.metadata.created_at.clone(),
            updated_at: issue.metadata.updated_at.clone(),
            custom_fields: issue.metadata.custom_fields.clone(),
            priority_label: priority_label(issue.metadata.priority, priority_levels),
            draft: issue.metadata.draft,
            deleted_at: issue.metadata.deleted_at.clone().unwrap_or_default(),
            is_org_issue: issue.metadata.is_org_issue,
            org_slug: issue.metadata.org_slug.clone().unwrap_or_default(),
            org_display_number: issue.metadata.org_display_number.unwrap_or(0),
        }),
    }
}

fn doc_to_proto(doc: &crate::item::entities::doc::Doc) -> Doc {
    Doc {
        slug: doc.slug.clone(),
        title: doc.title.clone(),
        content: doc.content.clone(),
        metadata: Some(DocMetadata {
            created_at: doc.metadata.created_at.clone(),
            updated_at: doc.metadata.updated_at.clone(),
            deleted_at: doc.metadata.deleted_at.clone().unwrap_or_default(),
            is_org_doc: doc.metadata.is_org_doc,
            org_slug: doc.metadata.org_slug.clone().unwrap_or_default(),
        }),
    }
}

fn project_info_to_proto(info: &ProjectInfo) -> proto::ProjectInfo {
    proto::ProjectInfo {
        path: info.path.clone(),
        first_accessed: info.first_accessed.clone(),
        last_accessed: info.last_accessed.clone(),
        issue_count: info.issue_count,
        doc_count: info.doc_count,
        initialized: info.initialized,
        name: info.name.clone().unwrap_or_default(),
        is_favorite: info.is_favorite,
        is_archived: info.is_archived,
        display_path: format_display_path(&info.path),
        organization_slug: info.organization_slug.clone().unwrap_or_default(),
        organization_name: info.organization_name.clone().unwrap_or_default(),
        user_title: info.user_title.clone().unwrap_or_default(),
        project_title: info.project_title.clone().unwrap_or_default(),
    }
}

fn org_info_to_proto(info: &OrganizationInfo) -> ProtoOrganization {
    ProtoOrganization {
        slug: info.slug.clone(),
        name: info.name.clone(),
        description: info.description.clone().unwrap_or_default(),
        created_at: info.created_at.clone(),
        updated_at: info.updated_at.clone(),
        project_count: info.project_count,
    }
}

fn org_inference_to_proto(result: &OrgInferenceResult) -> ProtoOrgInferenceResult {
    ProtoOrgInferenceResult {
        inferred_org_slug: result.inferred_org_slug.clone().unwrap_or_default(),
        inferred_org_name: result.inferred_org_name.clone().unwrap_or_default(),
        org_created: result.org_created,
        existing_org_slug: result.existing_org_slug.clone().unwrap_or_default(),
        has_mismatch: result.has_mismatch,
        message: result.message.clone().unwrap_or_default(),
    }
}

fn asset_info_to_proto(asset: &AssetInfo) -> Asset {
    Asset {
        filename: asset.filename.clone(),
        hash: asset.hash.clone(),
        size: asset.size,
        mime_type: asset.mime_type.clone(),
        is_shared: asset.is_shared,
        created_at: asset.created_at.clone(),
    }
}

fn user_to_proto(user: &crate::user::User) -> ProtoUser {
    ProtoUser {
        id: user.id.clone(),
        name: user.name.clone(),
        email: user.email.clone().unwrap_or_default(),
        git_usernames: user.git_usernames.clone(),
        created_at: user.created_at.clone(),
        updated_at: user.updated_at.clone(),
        deleted_at: user.deleted_at.clone().unwrap_or_default(),
    }
}

fn pr_to_proto(pr: &crate::item::entities::pr::PullRequest, priority_levels: u32) -> PullRequest {
    PullRequest {
        id: pr.id.clone(),
        display_number: pr.metadata.display_number,
        title: pr.title.clone(),
        description: pr.description.clone(),
        metadata: Some(PrMetadata {
            display_number: pr.metadata.display_number,
            status: pr.metadata.status.clone(),
            source_branch: pr.metadata.source_branch.clone(),
            target_branch: pr.metadata.target_branch.clone(),
            reviewers: pr.metadata.reviewers.clone(),
            priority: pr.metadata.priority as i32,
            priority_label: priority_label(pr.metadata.priority, priority_levels),
            created_at: pr.metadata.created_at.clone(),
            updated_at: pr.metadata.updated_at.clone(),
            merged_at: pr.metadata.merged_at.clone(),
            closed_at: pr.metadata.closed_at.clone(),
            custom_fields: pr.metadata.custom_fields.clone(),
            deleted_at: pr.metadata.deleted_at.clone().unwrap_or_default(),
        }),
    }
}
