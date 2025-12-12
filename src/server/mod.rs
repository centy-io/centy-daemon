use crate::config::{read_config, write_config, CentyConfig, CustomFieldDefinition as InternalCustomFieldDef, LlmConfig as InternalLlmConfig};
use crate::link::{
    create_link, delete_link, get_available_link_types, list_links,
    CreateLinkOptions, DeleteLinkOptions, TargetType,
};
use crate::llm::{
    self, get_effective_local_config, spawn_agent as llm_spawn_agent, read_work_session,
    record_work_session, clear_work_session, is_process_running, has_global_config, has_project_config,
    write_global_local_config, write_project_local_config, LlmAction, AgentType as InternalAgentType,
};
use crate::features::{
    get_compact, get_feature_status, get_instruction, list_uncompacted_issues,
    mark_issues_compacted, save_migration, update_compact,
};
use crate::migration::{create_registry, MigrationExecutor};
use crate::version::{compare_versions, daemon_version, SemVer, VersionComparison};
use crate::docs::{
    create_doc, delete_doc, duplicate_doc, get_doc, get_docs_by_slug, list_docs, move_doc, update_doc,
    CreateDocOptions, DuplicateDocOptions, MoveDocOptions, UpdateDocOptions,
};
use crate::issue::{
    create_issue, delete_issue, duplicate_issue, get_issue, get_issue_by_display_number,
    get_issues_by_uuid, list_issues, move_issue, priority_label, update_issue,
    CreateIssueOptions, DuplicateIssueOptions, MoveIssueOptions, UpdateIssueOptions,
    // Asset imports
    add_asset, delete_asset as delete_asset_fn, get_asset, list_assets, list_shared_assets,
    AssetInfo, AssetScope,
};
use crate::pr::{
    create_pr, delete_pr, get_pr, get_pr_by_display_number, get_prs_by_uuid, list_prs, update_pr,
    CreatePrOptions, UpdatePrOptions,
};
use crate::manifest::{read_manifest, ManagedFileType as InternalFileType, CentyManifest as InternalManifest};
use crate::reconciliation::{
    build_reconciliation_plan, execute_reconciliation, ReconciliationDecisions,
};
use crate::registry::{
    create_organization, delete_organization, get_organization, get_project_info, list_organizations,
    list_projects, set_project_archived, set_project_favorite, set_project_organization,
    track_project_async, untrack_project, update_organization, ListProjectsOptions, OrganizationInfo,
    ProjectInfo,
};
use crate::utils::{format_display_path, get_centy_path};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::watch;
use tonic::{Request, Response, Status};
use tracing::info;

// Import generated protobuf types
pub mod proto {
    #![allow(clippy::pedantic)]
    #![allow(clippy::all)]
    tonic::include_proto!("centy");
}

use proto::centy_daemon_server::CentyDaemon;
use proto::{InitRequest, InitResponse, GetReconciliationPlanRequest, ReconciliationPlan, ExecuteReconciliationRequest, CreateIssueRequest, CreateIssueResponse, GetIssueRequest, Issue, GetIssueByDisplayNumberRequest, GetIssuesByUuidRequest, GetIssuesByUuidResponse, IssueWithProject as ProtoIssueWithProject, ListIssuesRequest, ListIssuesResponse, UpdateIssueRequest, UpdateIssueResponse, DeleteIssueRequest, DeleteIssueResponse, MoveIssueRequest, MoveIssueResponse, DuplicateIssueRequest, DuplicateIssueResponse, GetNextIssueNumberRequest, GetNextIssueNumberResponse, GetManifestRequest, Manifest, GetConfigRequest, Config, LlmConfig, UpdateConfigRequest, UpdateConfigResponse, IsInitializedRequest, IsInitializedResponse, CreateDocRequest, CreateDocResponse, GetDocRequest, Doc, GetDocsBySlugRequest, GetDocsBySlugResponse, DocWithProject as ProtoDocWithProject, ListDocsRequest, ListDocsResponse, UpdateDocRequest, UpdateDocResponse, DeleteDocRequest, DeleteDocResponse, MoveDocRequest, MoveDocResponse, DuplicateDocRequest, DuplicateDocResponse, AddAssetRequest, AddAssetResponse, ListAssetsRequest, ListAssetsResponse, GetAssetRequest, GetAssetResponse, DeleteAssetRequest, DeleteAssetResponse, ListSharedAssetsRequest, ListProjectsRequest, ListProjectsResponse, RegisterProjectRequest, RegisterProjectResponse, UntrackProjectRequest, UntrackProjectResponse, GetProjectInfoRequest, GetProjectInfoResponse, SetProjectFavoriteRequest, SetProjectFavoriteResponse, SetProjectArchivedRequest, SetProjectArchivedResponse, SetProjectOrganizationRequest, SetProjectOrganizationResponse, CreateOrganizationRequest, CreateOrganizationResponse, ListOrganizationsRequest, ListOrganizationsResponse, GetOrganizationRequest, GetOrganizationResponse, UpdateOrganizationRequest, UpdateOrganizationResponse, DeleteOrganizationRequest, DeleteOrganizationResponse, Organization as ProtoOrganization, GetDaemonInfoRequest, DaemonInfo, GetProjectVersionRequest, ProjectVersionInfo, UpdateVersionRequest, UpdateVersionResponse, ShutdownRequest, ShutdownResponse, RestartRequest, RestartResponse, CreatePrRequest, CreatePrResponse, GetPrRequest, PullRequest, GetPrByDisplayNumberRequest, GetPrsByUuidRequest, GetPrsByUuidResponse, PrWithProject as ProtoPrWithProject, ListPrsRequest, ListPrsResponse, UpdatePrRequest, UpdatePrResponse, DeletePrRequest, DeletePrResponse, GetNextPrNumberRequest, GetNextPrNumberResponse, GetFeatureStatusRequest, GetFeatureStatusResponse, ListUncompactedIssuesRequest, ListUncompactedIssuesResponse, GetInstructionRequest, GetInstructionResponse, GetCompactRequest, GetCompactResponse, UpdateCompactRequest, UpdateCompactResponse, SaveMigrationRequest, SaveMigrationResponse, MarkIssuesCompactedRequest, MarkIssuesCompactedResponse, SpawnAgentRequest, SpawnAgentResponse, GetLlmWorkRequest, GetLlmWorkResponse, LlmWorkSession, ClearLlmWorkRequest, ClearLlmWorkResponse, GetLocalLlmConfigRequest, GetLocalLlmConfigResponse, UpdateLocalLlmConfigRequest, UpdateLocalLlmConfigResponse, FileInfo, FileType, CustomFieldDefinition, IssueMetadata, DocMetadata, Asset, PrMetadata, LocalLlmConfig, AgentConfig, AgentType, LinkTypeDefinition, CreateLinkRequest, CreateLinkResponse, DeleteLinkRequest, DeleteLinkResponse, ListLinksRequest, ListLinksResponse, GetAvailableLinkTypesRequest, GetAvailableLinkTypesResponse, Link as ProtoLink, LinkTargetType, LinkTypeInfo};

/// Signal type for daemon shutdown/restart
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShutdownSignal {
    None,
    Shutdown,
    Restart,
}

pub struct CentyDaemonService {
    shutdown_tx: Arc<watch::Sender<ShutdownSignal>>,
    exe_path: Option<PathBuf>,
}

impl CentyDaemonService {
    #[must_use] 
    pub fn new(shutdown_tx: Arc<watch::Sender<ShutdownSignal>>, exe_path: Option<PathBuf>) -> Self {
        Self { shutdown_tx, exe_path }
    }
}

#[tonic::async_trait]
impl CentyDaemon for CentyDaemonService {
    async fn init(&self, request: Request<InitRequest>) -> Result<Response<InitResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let decisions = req.decisions.map(|d| ReconciliationDecisions {
            restore: d.restore.into_iter().collect(),
            reset: d.reset.into_iter().collect(),
        }).unwrap_or_default();

        match execute_reconciliation(project_path, decisions, req.force).await {
            Ok(result) => Ok(Response::new(InitResponse {
                success: true,
                error: String::new(),
                created: result.created,
                restored: result.restored,
                reset: result.reset,
                skipped: result.skipped,
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(InitResponse {
                success: false,
                error: e.to_string(),
                created: vec![],
                restored: vec![],
                reset: vec![],
                skipped: vec![],
                manifest: None,
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
                    to_restore: plan.to_restore.into_iter().map(file_info_to_proto).collect(),
                    to_reset: plan.to_reset.into_iter().map(file_info_to_proto).collect(),
                    up_to_date: plan.up_to_date.into_iter().map(file_info_to_proto).collect(),
                    user_files: plan.user_files.into_iter().map(file_info_to_proto).collect(),
                    needs_decisions,
                }))
            },
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

        let decisions = req.decisions.map(|d| ReconciliationDecisions {
            restore: d.restore.into_iter().collect(),
            reset: d.reset.into_iter().collect(),
        }).unwrap_or_default();

        match execute_reconciliation(project_path, decisions, false).await {
            Ok(result) => Ok(Response::new(InitResponse {
                success: true,
                error: String::new(),
                created: result.created,
                restored: result.restored,
                reset: result.reset,
                skipped: result.skipped,
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(InitResponse {
                success: false,
                error: e.to_string(),
                created: vec![],
                restored: vec![],
                reset: vec![],
                skipped: vec![],
                manifest: None,
            })),
        }
    }

    async fn create_issue(
        &self,
        request: Request<CreateIssueRequest>,
    ) -> Result<Response<CreateIssueResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Convert int32 priority: 0 means use default, otherwise use the value
        let options = CreateIssueOptions {
            title: req.title,
            description: req.description,
            priority: if req.priority == 0 { None } else { Some(req.priority as u32) },
            status: if req.status.is_empty() { None } else { Some(req.status) },
            custom_fields: req.custom_fields,
            template: if req.template.is_empty() { None } else { Some(req.template) },
        };

        match create_issue(project_path, options).await {
            #[allow(deprecated)]
            Ok(result) => Ok(Response::new(CreateIssueResponse {
                success: true,
                error: String::new(),
                id: result.id.clone(),
                display_number: result.display_number,
                issue_number: result.issue_number, // Legacy
                created_files: result.created_files,
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(CreateIssueResponse {
                success: false,
                error: e.to_string(),
                id: String::new(),
                display_number: 0,
                issue_number: String::new(),
                created_files: vec![],
                manifest: None,
            })),
        }
    }

    async fn get_issue(
        &self,
        request: Request<GetIssueRequest>,
    ) -> Result<Response<Issue>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        match get_issue(project_path, &req.issue_id).await {
            Ok(issue) => Ok(Response::new(issue_to_proto(&issue, priority_levels))),
            Err(e) => Err(Status::not_found(e.to_string())),
        }
    }

    async fn get_issue_by_display_number(
        &self,
        request: Request<GetIssueByDisplayNumberRequest>,
    ) -> Result<Response<Issue>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        match get_issue_by_display_number(project_path, req.display_number).await {
            Ok(issue) => Ok(Response::new(issue_to_proto(&issue, priority_levels))),
            Err(e) => Err(Status::not_found(e.to_string())),
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

    async fn list_issues(
        &self,
        request: Request<ListIssuesRequest>,
    ) -> Result<Response<ListIssuesResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        let status_filter = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        // Convert int32 priority filter: 0 means no filter
        let priority_filter = if req.priority == 0 { None } else { Some(req.priority as u32) };

        match list_issues(project_path, status_filter, priority_filter).await {
            Ok(issues) => {
                let total_count = issues.len() as i32;
                Ok(Response::new(ListIssuesResponse {
                    issues: issues.into_iter().map(|i| issue_to_proto(&i, priority_levels)).collect(),
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

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        // Convert int32 priority: 0 means don't update, otherwise use the value
        let options = UpdateIssueOptions {
            title: if req.title.is_empty() { None } else { Some(req.title) },
            description: if req.description.is_empty() { None } else { Some(req.description) },
            status: if req.status.is_empty() { None } else { Some(req.status) },
            priority: if req.priority == 0 { None } else { Some(req.priority as u32) },
            custom_fields: req.custom_fields,
        };

        match update_issue(project_path, &req.issue_id, options).await {
            Ok(result) => Ok(Response::new(UpdateIssueResponse {
                success: true,
                error: String::new(),
                issue: Some(issue_to_proto(&result.issue, priority_levels)),
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(UpdateIssueResponse {
                success: false,
                error: e.to_string(),
                issue: None,
                manifest: None,
            })),
        }
    }

    async fn delete_issue(
        &self,
        request: Request<DeleteIssueRequest>,
    ) -> Result<Response<DeleteIssueResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match delete_issue(project_path, &req.issue_id).await {
            Ok(result) => Ok(Response::new(DeleteIssueResponse {
                success: true,
                error: String::new(),
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(DeleteIssueResponse {
                success: false,
                error: e.to_string(),
                manifest: None,
            })),
        }
    }

    async fn move_issue(
        &self,
        request: Request<MoveIssueRequest>,
    ) -> Result<Response<MoveIssueResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.source_project_path.clone());
        track_project_async(req.target_project_path.clone());

        // Read target config for priority_levels
        let target_config = read_config(Path::new(&req.target_project_path)).await.ok().flatten();
        let priority_levels = target_config.as_ref().map_or(3, |c| c.priority_levels);

        let options = MoveIssueOptions {
            source_project_path: PathBuf::from(&req.source_project_path),
            target_project_path: PathBuf::from(&req.target_project_path),
            issue_id: req.issue_id,
        };

        match move_issue(options).await {
            Ok(result) => Ok(Response::new(MoveIssueResponse {
                success: true,
                error: String::new(),
                issue: Some(issue_to_proto(&result.issue, priority_levels)),
                old_display_number: result.old_display_number,
                source_manifest: Some(manifest_to_proto(&result.source_manifest)),
                target_manifest: Some(manifest_to_proto(&result.target_manifest)),
            })),
            Err(e) => Ok(Response::new(MoveIssueResponse {
                success: false,
                error: e.to_string(),
                issue: None,
                old_display_number: 0,
                source_manifest: None,
                target_manifest: None,
            })),
        }
    }

    async fn duplicate_issue(
        &self,
        request: Request<DuplicateIssueRequest>,
    ) -> Result<Response<DuplicateIssueResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.source_project_path.clone());
        track_project_async(req.target_project_path.clone());

        // Read target config for priority_levels
        let target_config = read_config(Path::new(&req.target_project_path)).await.ok().flatten();
        let priority_levels = target_config.as_ref().map_or(3, |c| c.priority_levels);

        let options = DuplicateIssueOptions {
            source_project_path: PathBuf::from(&req.source_project_path),
            target_project_path: PathBuf::from(&req.target_project_path),
            issue_id: req.issue_id,
            new_title: if req.new_title.is_empty() { None } else { Some(req.new_title) },
        };

        match duplicate_issue(options).await {
            Ok(result) => Ok(Response::new(DuplicateIssueResponse {
                success: true,
                error: String::new(),
                issue: Some(issue_to_proto(&result.issue, priority_levels)),
                original_issue_id: result.original_issue_id,
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(DuplicateIssueResponse {
                success: false,
                error: e.to_string(),
                issue: None,
                original_issue_id: String::new(),
                manifest: None,
            })),
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
        match crate::issue::create::get_next_issue_number(&issues_path).await {
            Ok(issue_number) => Ok(Response::new(GetNextIssueNumberResponse { issue_number })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_manifest(
        &self,
        request: Request<GetManifestRequest>,
    ) -> Result<Response<Manifest>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match read_manifest(project_path).await {
            Ok(Some(manifest)) => Ok(Response::new(manifest_to_proto(&manifest))),
            Ok(None) => Err(Status::not_found("Manifest not found")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_config(
        &self,
        request: Request<GetConfigRequest>,
    ) -> Result<Response<Config>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match read_config(project_path).await {
            Ok(Some(config)) => Ok(Response::new(config_to_proto(&config))),
            Ok(None) => Ok(Response::new(Config {
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
                    update_status_on_start: false,
                    allow_direct_edits: false,
                }),
                custom_link_types: vec![],
            })),
            Err(e) => Err(Status::internal(e.to_string())),
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

        let options = CreateDocOptions {
            title: req.title,
            content: req.content,
            slug: if req.slug.is_empty() { None } else { Some(req.slug) },
            template: if req.template.is_empty() { None } else { Some(req.template) },
        };

        match create_doc(project_path, options).await {
            Ok(result) => Ok(Response::new(CreateDocResponse {
                success: true,
                error: String::new(),
                slug: result.slug,
                created_file: result.created_file,
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(CreateDocResponse {
                success: false,
                error: e.to_string(),
                slug: String::new(),
                created_file: String::new(),
                manifest: None,
            })),
        }
    }

    async fn get_doc(
        &self,
        request: Request<GetDocRequest>,
    ) -> Result<Response<Doc>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match get_doc(project_path, &req.slug).await {
            Ok(doc) => Ok(Response::new(doc_to_proto(&doc))),
            Err(e) => Err(Status::not_found(e.to_string())),
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

        match list_docs(project_path).await {
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

        let options = UpdateDocOptions {
            title: if req.title.is_empty() { None } else { Some(req.title) },
            content: if req.content.is_empty() { None } else { Some(req.content) },
            new_slug: if req.new_slug.is_empty() { None } else { Some(req.new_slug) },
        };

        match update_doc(project_path, &req.slug, options).await {
            Ok(result) => Ok(Response::new(UpdateDocResponse {
                success: true,
                error: String::new(),
                doc: Some(doc_to_proto(&result.doc)),
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(UpdateDocResponse {
                success: false,
                error: e.to_string(),
                doc: None,
                manifest: None,
            })),
        }
    }

    async fn delete_doc(
        &self,
        request: Request<DeleteDocRequest>,
    ) -> Result<Response<DeleteDocResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match delete_doc(project_path, &req.slug).await {
            Ok(result) => Ok(Response::new(DeleteDocResponse {
                success: true,
                error: String::new(),
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(DeleteDocResponse {
                success: false,
                error: e.to_string(),
                manifest: None,
            })),
        }
    }

    async fn move_doc(
        &self,
        request: Request<MoveDocRequest>,
    ) -> Result<Response<MoveDocResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.source_project_path.clone());
        track_project_async(req.target_project_path.clone());

        let options = MoveDocOptions {
            source_project_path: PathBuf::from(&req.source_project_path),
            target_project_path: PathBuf::from(&req.target_project_path),
            slug: req.slug.clone(),
            new_slug: if req.new_slug.is_empty() { None } else { Some(req.new_slug) },
        };

        match move_doc(options).await {
            Ok(result) => Ok(Response::new(MoveDocResponse {
                success: true,
                error: String::new(),
                doc: Some(doc_to_proto(&result.doc)),
                old_slug: result.old_slug,
                source_manifest: Some(manifest_to_proto(&result.source_manifest)),
                target_manifest: Some(manifest_to_proto(&result.target_manifest)),
            })),
            Err(e) => Ok(Response::new(MoveDocResponse {
                success: false,
                error: e.to_string(),
                doc: None,
                old_slug: req.slug,
                source_manifest: None,
                target_manifest: None,
            })),
        }
    }

    async fn duplicate_doc(
        &self,
        request: Request<DuplicateDocRequest>,
    ) -> Result<Response<DuplicateDocResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.source_project_path.clone());
        track_project_async(req.target_project_path.clone());

        let options = DuplicateDocOptions {
            source_project_path: PathBuf::from(&req.source_project_path),
            target_project_path: PathBuf::from(&req.target_project_path),
            slug: req.slug.clone(),
            new_slug: if req.new_slug.is_empty() { None } else { Some(req.new_slug) },
            new_title: if req.new_title.is_empty() { None } else { Some(req.new_title) },
        };

        match duplicate_doc(options).await {
            Ok(result) => Ok(Response::new(DuplicateDocResponse {
                success: true,
                error: String::new(),
                doc: Some(doc_to_proto(&result.doc)),
                original_slug: result.original_slug,
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(DuplicateDocResponse {
                success: false,
                error: e.to_string(),
                doc: None,
                original_slug: req.slug,
                manifest: None,
            })),
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
            Err(e) => Ok(Response::new(AddAssetResponse {
                success: false,
                error: e.to_string(),
                asset: None,
                path: String::new(),
                manifest: None,
            })),
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

        let issue_id = if req.issue_id.is_empty() {
            None
        } else {
            Some(req.issue_id.as_str())
        };

        match delete_asset_fn(project_path, issue_id, &req.filename, req.is_shared).await {
            Ok(result) => {
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
            Err(e) => Ok(Response::new(DeleteAssetResponse {
                success: false,
                error: e.to_string(),
                filename: String::new(),
                was_shared: false,
                manifest: None,
            })),
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
        };
        match list_projects(opts).await {
            Ok(projects) => {
                let total_count = projects.len() as i32;
                Ok(Response::new(ListProjectsResponse {
                    projects: projects.into_iter().map(|p| project_info_to_proto(&p)).collect(),
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

        // Track the project (this creates or updates the entry)
        if let Err(e) = crate::registry::track_project(&req.project_path).await {
            return Ok(Response::new(RegisterProjectResponse {
                success: false,
                error: e.to_string(),
                project: None,
            }));
        }

        // Get the project info
        match get_project_info(&req.project_path).await {
            Ok(Some(info)) => Ok(Response::new(RegisterProjectResponse {
                success: true,
                error: String::new(),
                project: Some(project_info_to_proto(&info)),
            })),
            Ok(None) => Ok(Response::new(RegisterProjectResponse {
                success: false,
                error: "Failed to retrieve project after registration".to_string(),
                project: None,
            })),
            Err(e) => Ok(Response::new(RegisterProjectResponse {
                success: false,
                error: e.to_string(),
                project: None,
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

    // ============ Organization RPCs ============

    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<CreateOrganizationResponse>, Status> {
        let req = request.into_inner();
        let slug = if req.slug.is_empty() { None } else { Some(req.slug.as_str()) };
        let description = if req.description.is_empty() { None } else { Some(req.description.as_str()) };

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
        let name = if req.name.is_empty() { None } else { Some(req.name.as_str()) };
        let description = if req.description.is_empty() { None } else { Some(req.description.as_str()) };
        let new_slug = if req.new_slug.is_empty() { None } else { Some(req.new_slug.as_str()) };

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
        let daemon_ver = daemon_version();
        let registry = create_registry();
        let binary_path = std::env::current_exe()
            .map(|p| format_display_path(&p.to_string_lossy()))
            .unwrap_or_default();

        Ok(Response::new(DaemonInfo {
            version: daemon_ver.to_string(),
            available_versions: registry.available_versions(),
            binary_path,
        }))
    }

    async fn get_project_version(
        &self,
        request: Request<GetProjectVersionRequest>,
    ) -> Result<Response<ProjectVersionInfo>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let config = read_config(project_path).await.ok().flatten();
        let project_ver_str = config
            .as_ref()
            .and_then(|c| c.version.clone())
            .unwrap_or_else(|| crate::utils::CENTY_VERSION.to_string());

        let project_ver = match SemVer::parse(&project_ver_str) {
            Ok(v) => v,
            Err(e) => return Err(Status::invalid_argument(e.to_string())),
        };
        let daemon_ver = daemon_version();

        let comparison = compare_versions(&project_ver, &daemon_ver);
        let (comparison_str, degraded) = match comparison {
            VersionComparison::Equal => ("equal", false),
            VersionComparison::ProjectBehind => ("project_behind", false),
            VersionComparison::ProjectAhead => ("project_ahead", true),
        };

        Ok(Response::new(ProjectVersionInfo {
            project_version: project_ver_str,
            daemon_version: daemon_ver.to_string(),
            comparison: comparison_str.to_string(),
            degraded_mode: degraded,
        }))
    }

    async fn update_version(
        &self,
        request: Request<UpdateVersionRequest>,
    ) -> Result<Response<UpdateVersionResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let target = match SemVer::parse(&req.target_version) {
            Ok(v) => v,
            Err(e) => {
                return Ok(Response::new(UpdateVersionResponse {
                    success: false,
                    error: format!("Invalid target version: {e}"),
                    from_version: String::new(),
                    to_version: String::new(),
                    migrations_applied: vec![],
                }));
            }
        };

        let registry = create_registry();
        let executor = MigrationExecutor::new(registry);

        match executor.migrate(project_path, &target).await {
            Ok(result) => Ok(Response::new(UpdateVersionResponse {
                success: result.success,
                error: result.error.unwrap_or_default(),
                from_version: result.from_version,
                to_version: result.to_version,
                migrations_applied: result.migrations_applied,
            })),
            Err(e) => Ok(Response::new(UpdateVersionResponse {
                success: false,
                error: e.to_string(),
                from_version: String::new(),
                to_version: String::new(),
                migrations_applied: vec![],
            })),
        }
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

        let options = CreatePrOptions {
            title: req.title,
            description: req.description,
            source_branch: if req.source_branch.is_empty() { None } else { Some(req.source_branch) },
            target_branch: if req.target_branch.is_empty() { None } else { Some(req.target_branch) },
            reviewers: req.reviewers,
            priority: if req.priority == 0 { None } else { Some(req.priority as u32) },
            status: if req.status.is_empty() { None } else { Some(req.status) },
            custom_fields: req.custom_fields,
            template: if req.template.is_empty() { None } else { Some(req.template) },
        };

        match create_pr(project_path, options).await {
            Ok(result) => Ok(Response::new(CreatePrResponse {
                success: true,
                error: String::new(),
                id: result.id,
                display_number: result.display_number,
                created_files: result.created_files,
                manifest: Some(manifest_to_proto(&result.manifest)),
                detected_source_branch: result.detected_source_branch,
            })),
            Err(e) => Ok(Response::new(CreatePrResponse {
                success: false,
                error: e.to_string(),
                id: String::new(),
                display_number: 0,
                created_files: vec![],
                manifest: None,
                detected_source_branch: String::new(),
            })),
        }
    }

    async fn get_pr(
        &self,
        request: Request<GetPrRequest>,
    ) -> Result<Response<PullRequest>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        match get_pr(project_path, &req.pr_id).await {
            Ok(pr) => Ok(Response::new(pr_to_proto(&pr, priority_levels))),
            Err(e) => Err(Status::not_found(e.to_string())),
        }
    }

    async fn get_pr_by_display_number(
        &self,
        request: Request<GetPrByDisplayNumberRequest>,
    ) -> Result<Response<PullRequest>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        match get_pr_by_display_number(project_path, req.display_number).await {
            Ok(pr) => Ok(Response::new(pr_to_proto(&pr, priority_levels))),
            Err(e) => Err(Status::not_found(e.to_string())),
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

        let status_filter = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        let source_filter = if req.source_branch.is_empty() { None } else { Some(req.source_branch.as_str()) };
        let target_filter = if req.target_branch.is_empty() { None } else { Some(req.target_branch.as_str()) };
        let priority_filter = if req.priority == 0 { None } else { Some(req.priority as u32) };

        match list_prs(project_path, status_filter, source_filter, target_filter, priority_filter).await {
            Ok(prs) => {
                let total_count = prs.len() as i32;
                Ok(Response::new(ListPrsResponse {
                    prs: prs.into_iter().map(|p| pr_to_proto(&p, priority_levels)).collect(),
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

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        let options = UpdatePrOptions {
            title: if req.title.is_empty() { None } else { Some(req.title) },
            description: if req.description.is_empty() { None } else { Some(req.description) },
            status: if req.status.is_empty() { None } else { Some(req.status) },
            source_branch: if req.source_branch.is_empty() { None } else { Some(req.source_branch) },
            target_branch: if req.target_branch.is_empty() { None } else { Some(req.target_branch) },
            reviewers: if req.reviewers.is_empty() { None } else { Some(req.reviewers) },
            priority: if req.priority == 0 { None } else { Some(req.priority as u32) },
            custom_fields: req.custom_fields,
        };

        match update_pr(project_path, &req.pr_id, options).await {
            Ok(result) => Ok(Response::new(UpdatePrResponse {
                success: true,
                error: String::new(),
                pr: Some(pr_to_proto(&result.pr, priority_levels)),
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(UpdatePrResponse {
                success: false,
                error: e.to_string(),
                pr: None,
                manifest: None,
            })),
        }
    }

    async fn delete_pr(
        &self,
        request: Request<DeletePrRequest>,
    ) -> Result<Response<DeletePrResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match delete_pr(project_path, &req.pr_id).await {
            Ok(result) => Ok(Response::new(DeletePrResponse {
                success: true,
                error: String::new(),
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(DeletePrResponse {
                success: false,
                error: e.to_string(),
                manifest: None,
            })),
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

        match crate::pr::reconcile::get_next_pr_display_number(&prs_path).await {
            Ok(next_number) => Ok(Response::new(GetNextPrNumberResponse { next_number })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // ============ Features RPCs ============

    async fn get_feature_status(
        &self,
        request: Request<GetFeatureStatusRequest>,
    ) -> Result<Response<GetFeatureStatusResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match get_feature_status(project_path).await {
            Ok(status) => Ok(Response::new(GetFeatureStatusResponse {
                initialized: status.initialized,
                has_compact: status.has_compact,
                has_instruction: status.has_instruction,
                migration_count: status.migration_count,
                uncompacted_count: status.uncompacted_count,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_uncompacted_issues(
        &self,
        request: Request<ListUncompactedIssuesRequest>,
    ) -> Result<Response<ListUncompactedIssuesResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

        match list_uncompacted_issues(project_path).await {
            Ok(issues) => {
                let total_count = issues.len() as i32;
                Ok(Response::new(ListUncompactedIssuesResponse {
                    issues: issues.into_iter().map(|i| issue_to_proto(&i, priority_levels)).collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_instruction(
        &self,
        request: Request<GetInstructionRequest>,
    ) -> Result<Response<GetInstructionResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match get_instruction(project_path).await {
            Ok(content) => Ok(Response::new(GetInstructionResponse { content })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_compact(
        &self,
        request: Request<GetCompactRequest>,
    ) -> Result<Response<GetCompactResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match get_compact(project_path).await {
            Ok(content) => Ok(Response::new(GetCompactResponse {
                exists: content.is_some(),
                content: content.unwrap_or_default(),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_compact(
        &self,
        request: Request<UpdateCompactRequest>,
    ) -> Result<Response<UpdateCompactResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match update_compact(project_path, &req.content).await {
            Ok(()) => Ok(Response::new(UpdateCompactResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(UpdateCompactResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn save_migration(
        &self,
        request: Request<SaveMigrationRequest>,
    ) -> Result<Response<SaveMigrationResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match save_migration(project_path, &req.content).await {
            Ok((filename, path)) => Ok(Response::new(SaveMigrationResponse {
                success: true,
                error: String::new(),
                filename,
                path,
            })),
            Err(e) => Ok(Response::new(SaveMigrationResponse {
                success: false,
                error: e.to_string(),
                filename: String::new(),
                path: String::new(),
            })),
        }
    }

    async fn mark_issues_compacted(
        &self,
        request: Request<MarkIssuesCompactedRequest>,
    ) -> Result<Response<MarkIssuesCompactedResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match mark_issues_compacted(project_path, &req.issue_ids).await {
            Ok(marked_count) => Ok(Response::new(MarkIssuesCompactedResponse {
                success: true,
                error: String::new(),
                marked_count,
            })),
            Err(e) => Ok(Response::new(MarkIssuesCompactedResponse {
                success: false,
                error: e.to_string(),
                marked_count: 0,
            })),
        }
    }

    // ============ LLM Agent RPCs ============

    #[allow(clippy::too_many_lines)]
    async fn spawn_agent(
        &self,
        request: Request<SpawnAgentRequest>,
    ) -> Result<Response<SpawnAgentResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Parse action
        let action = match LlmAction::from_proto(req.action) {
            Some(a) => a,
            None => {
                return Ok(Response::new(SpawnAgentResponse {
                    success: false,
                    error: "Invalid action. Must be 1 (plan) or 2 (implement).".to_string(),
                    agent_name: String::new(),
                    issue_id: String::new(),
                    display_number: 0,
                    prompt_preview: String::new(),
                }));
            }
        };

        // Resolve issue - try parsing as display number first, then as UUID
        let issue = if let Ok(display_num) = req.issue_id.parse::<u32>() {
            match get_issue_by_display_number(project_path, display_num).await {
                Ok(i) => i,
                Err(e) => {
                    return Ok(Response::new(SpawnAgentResponse {
                        success: false,
                        error: format!("Issue not found: {e}"),
                        agent_name: String::new(),
                        issue_id: String::new(),
                        display_number: 0,
                        prompt_preview: String::new(),
                    }));
                }
            }
        } else {
            match get_issue(project_path, &req.issue_id).await {
                Ok(i) => i,
                Err(e) => {
                    return Ok(Response::new(SpawnAgentResponse {
                        success: false,
                        error: format!("Issue not found: {e}"),
                        agent_name: String::new(),
                        issue_id: String::new(),
                        display_number: 0,
                        prompt_preview: String::new(),
                    }));
                }
            }
        };

        // Get effective config
        let llm_config = match get_effective_local_config(Some(project_path)).await {
            Ok(c) => c,
            Err(e) => {
                return Ok(Response::new(SpawnAgentResponse {
                    success: false,
                    error: format!("Failed to load LLM config: {e}"),
                    agent_name: String::new(),
                    issue_id: String::new(),
                    display_number: 0,
                    prompt_preview: String::new(),
                }));
            }
        };

        // Get project config for priority levels
        let project_config = read_config(project_path).await.ok().flatten().unwrap_or_default();
        let priority_levels = project_config.priority_levels;

        // Resolve agent name
        let agent_name = if req.agent_name.is_empty() {
            None
        } else {
            Some(req.agent_name.as_str())
        };

        // Spawn agent
        match llm_spawn_agent(
            project_path,
            &llm_config,
            &issue,
            action,
            agent_name,
            req.extra_args,
            priority_levels,
        )
        .await
        {
            Ok(result) => {
                // Record work session
                let _ = record_work_session(
                    project_path,
                    &issue.id,
                    issue.metadata.display_number,
                    &issue.title,
                    &result.agent_name,
                    action,
                    result.pid,
                )
                .await;

                Ok(Response::new(SpawnAgentResponse {
                    success: true,
                    error: String::new(),
                    agent_name: result.agent_name,
                    issue_id: issue.id,
                    display_number: issue.metadata.display_number,
                    prompt_preview: result.prompt_preview,
                }))
            }
            Err(e) => Ok(Response::new(SpawnAgentResponse {
                success: false,
                error: e.to_string(),
                agent_name: String::new(),
                issue_id: String::new(),
                display_number: 0,
                prompt_preview: String::new(),
            })),
        }
    }

    async fn get_llm_work(
        &self,
        request: Request<GetLlmWorkRequest>,
    ) -> Result<Response<GetLlmWorkResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        match read_work_session(project_path).await {
            Ok(Some(session)) => {
                // Check if process is still running
                let pid = session.pid.filter(|&p| is_process_running(p));

                Ok(Response::new(GetLlmWorkResponse {
                    has_active_work: true,
                    session: Some(LlmWorkSession {
                        issue_id: session.issue_id,
                        display_number: session.display_number,
                        issue_title: session.issue_title,
                        agent_name: session.agent_name,
                        action: match session.action.as_str() {
                            "plan" => 1,
                            "implement" => 2,
                            _ => 0,
                        },
                        started_at: session.started_at,
                        pid: pid.unwrap_or(0),
                    }),
                }))
            }
            Ok(None) => Ok(Response::new(GetLlmWorkResponse {
                has_active_work: false,
                session: None,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn clear_llm_work(
        &self,
        request: Request<ClearLlmWorkRequest>,
    ) -> Result<Response<ClearLlmWorkResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        match clear_work_session(project_path).await {
            Ok(()) => Ok(Response::new(ClearLlmWorkResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(ClearLlmWorkResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn get_local_llm_config(
        &self,
        request: Request<GetLocalLlmConfigRequest>,
    ) -> Result<Response<GetLocalLlmConfigResponse>, Status> {
        let req = request.into_inner();
        let project_path = if req.project_path.is_empty() {
            None
        } else {
            Some(Path::new(&req.project_path))
        };

        let has_global = has_global_config().await;
        let has_project = project_path.is_some_and(has_project_config);

        match get_effective_local_config(project_path).await {
            Ok(config) => Ok(Response::new(GetLocalLlmConfigResponse {
                config: Some(local_llm_config_to_proto(&config)),
                has_project_config: has_project,
                has_global_config: has_global,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_local_llm_config(
        &self,
        request: Request<UpdateLocalLlmConfigRequest>,
    ) -> Result<Response<UpdateLocalLlmConfigResponse>, Status> {
        let req = request.into_inner();

        let config = match req.config {
            Some(c) => proto_to_local_llm_config(&c),
            None => {
                return Ok(Response::new(UpdateLocalLlmConfigResponse {
                    success: false,
                    error: "Config is required".to_string(),
                    config: None,
                }));
            }
        };

        let result = if req.project_path.is_empty() {
            write_global_local_config(&config).await
        } else {
            let project_path = Path::new(&req.project_path);
            write_project_local_config(project_path, &config).await
        };

        match result {
            Ok(()) => Ok(Response::new(UpdateLocalLlmConfigResponse {
                success: true,
                error: String::new(),
                config: Some(local_llm_config_to_proto(&config)),
            })),
            Err(e) => Ok(Response::new(UpdateLocalLlmConfigResponse {
                success: false,
                error: e.to_string(),
                config: None,
            })),
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
            Ok(result) => Ok(Response::new(CreateLinkResponse {
                success: true,
                error: String::new(),
                created_link: Some(internal_link_to_proto(&result.created_link)),
                inverse_link: Some(internal_link_to_proto(&result.inverse_link)),
            })),
            Err(e) => Ok(Response::new(CreateLinkResponse {
                success: false,
                error: e.to_string(),
                created_link: None,
                inverse_link: None,
            })),
        }
    }

    async fn delete_link(
        &self,
        request: Request<DeleteLinkRequest>,
    ) -> Result<Response<DeleteLinkResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

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
            link_type: if req.link_type.is_empty() { None } else { Some(req.link_type) },
        };

        match delete_link(project_path, options, &custom_types).await {
            Ok(result) => Ok(Response::new(DeleteLinkResponse {
                success: true,
                error: String::new(),
                deleted_count: result.deleted_count,
            })),
            Err(e) => Ok(Response::new(DeleteLinkResponse {
                success: false,
                error: e.to_string(),
                deleted_count: 0,
            })),
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
                links: links_file.links.iter().map(internal_link_to_proto).collect(),
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
    }
}

fn proto_to_config(proto: &Config) -> CentyConfig {
    let llm_config = proto.llm.as_ref().map(|l| InternalLlmConfig {
        auto_close_on_complete: l.auto_close_on_complete,
        update_status_on_start: l.update_status_on_start,
        allow_direct_edits: l.allow_direct_edits,
    }).unwrap_or_default();

    CentyConfig {
        version: if proto.version.is_empty() { None } else { Some(proto.version.clone()) },
        priority_levels: proto.priority_levels as u32,
        custom_fields: proto
            .custom_fields
            .iter()
            .map(|f| InternalCustomFieldDef {
                name: f.name.clone(),
                field_type: f.field_type.clone(),
                required: f.required,
                default_value: if f.default_value.is_empty() { None } else { Some(f.default_value.clone()) },
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
    let hex_color_regex = regex::Regex::new(r"^#([0-9A-Fa-f]{3}|[0-9A-Fa-f]{6})$").unwrap();
    for (state, color) in &config.state_colors {
        if !hex_color_regex.is_match(color) {
            return Err(format!(
                "invalid color '{color}' for state '{state}': must be hex format (#RGB or #RRGGBB)"
            ));
        }
    }
    for (priority, color) in &config.priority_colors {
        if !hex_color_regex.is_match(color) {
            return Err(format!(
                "invalid color '{color}' for priority '{priority}': must be hex format (#RGB or #RRGGBB)"
            ));
        }
    }

    Ok(())
}

#[allow(deprecated)]
fn issue_to_proto(issue: &crate::issue::Issue, priority_levels: u32) -> Issue {
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
            compacted: issue.metadata.compacted,
            compacted_at: issue.metadata.compacted_at.clone().unwrap_or_default(),
        }),
    }
}

fn doc_to_proto(doc: &crate::docs::Doc) -> Doc {
    Doc {
        slug: doc.slug.clone(),
        title: doc.title.clone(),
        content: doc.content.clone(),
        metadata: Some(DocMetadata {
            created_at: doc.metadata.created_at.clone(),
            updated_at: doc.metadata.updated_at.clone(),
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

fn pr_to_proto(pr: &crate::pr::PullRequest, priority_levels: u32) -> PullRequest {
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
        }),
    }
}

fn local_llm_config_to_proto(config: &llm::LocalLlmConfig) -> LocalLlmConfig {
    LocalLlmConfig {
        default_agent: config.default_agent.clone().unwrap_or_default(),
        agents: config
            .agents
            .iter()
            .map(|a| AgentConfig {
                agent_type: match a.agent_type {
                    InternalAgentType::Claude => AgentType::Claude as i32,
                    InternalAgentType::Gemini => AgentType::Gemini as i32,
                    InternalAgentType::Codex => AgentType::Codex as i32,
                    InternalAgentType::Opencode => AgentType::Opencode as i32,
                    InternalAgentType::Custom => AgentType::Custom as i32,
                },
                name: a.name.clone(),
                command: a.command.clone(),
                default_args: a.default_args.clone(),
                plan_template: a.plan_template.clone().unwrap_or_default(),
                implement_template: a.implement_template.clone().unwrap_or_default(),
            })
            .collect(),
        env_vars: config.env_vars.clone(),
    }
}

fn proto_to_local_llm_config(proto: &LocalLlmConfig) -> llm::LocalLlmConfig {
    llm::LocalLlmConfig {
        default_agent: if proto.default_agent.is_empty() {
            None
        } else {
            Some(proto.default_agent.clone())
        },
        agents: proto
            .agents
            .iter()
            .map(|a| llm::AgentConfig {
                agent_type: match a.agent_type {
                    1 => InternalAgentType::Claude,
                    2 => InternalAgentType::Gemini,
                    3 => InternalAgentType::Codex,
                    4 => InternalAgentType::Opencode,
                    _ => InternalAgentType::Custom,
                },
                name: a.name.clone(),
                command: a.command.clone(),
                default_args: a.default_args.clone(),
                plan_template: if a.plan_template.is_empty() {
                    None
                } else {
                    Some(a.plan_template.clone())
                },
                implement_template: if a.implement_template.is_empty() {
                    None
                } else {
                    Some(a.implement_template.clone())
                },
            })
            .collect(),
        env_vars: proto.env_vars.clone(),
    }
}
