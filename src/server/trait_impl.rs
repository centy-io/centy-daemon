use crate::metrics::{generate_request_id, OperationTimer};
use tonic::{Request, Response, Status};
use tracing::instrument;

use super::handlers;
use super::proto::centy_daemon_server::CentyDaemon;
use super::proto::*;
use super::CentyDaemonService;

#[tonic::async_trait]
impl CentyDaemon for CentyDaemonService {
    #[instrument(
        name = "grpc.init",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn init(&self, request: Request<InitRequest>) -> Result<Response<InitResponse>, Status> {
        let _timer = OperationTimer::new("init");
        handlers::init::init(request.into_inner()).await
    }

    async fn get_reconciliation_plan(
        &self,
        request: Request<GetReconciliationPlanRequest>,
    ) -> Result<Response<ReconciliationPlan>, Status> {
        handlers::reconciliation::get_reconciliation_plan(request.into_inner()).await
    }

    async fn execute_reconciliation(
        &self,
        request: Request<ExecuteReconciliationRequest>,
    ) -> Result<Response<InitResponse>, Status> {
        handlers::reconciliation::execute_reconciliation_handler(request.into_inner()).await
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
        handlers::issue_create::create_issue(request.into_inner()).await
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
        handlers::issue_read::get_issue(request.into_inner()).await
    }

    async fn get_issue_by_display_number(
        &self,
        request: Request<GetIssueByDisplayNumberRequest>,
    ) -> Result<Response<GetIssueResponse>, Status> {
        handlers::issue_read::get_issue_by_display_number(request.into_inner()).await
    }

    async fn get_issues_by_uuid(
        &self,
        request: Request<GetIssuesByUuidRequest>,
    ) -> Result<Response<GetIssuesByUuidResponse>, Status> {
        handlers::issue_by_uuid::get_issues_by_uuid(request.into_inner()).await
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
        handlers::issue_list::list_issues(request.into_inner()).await
    }

    async fn update_issue(
        &self,
        request: Request<UpdateIssueRequest>,
    ) -> Result<Response<UpdateIssueResponse>, Status> {
        handlers::issue_update::update_issue(request.into_inner()).await
    }

    async fn delete_issue(
        &self,
        request: Request<DeleteIssueRequest>,
    ) -> Result<Response<DeleteIssueResponse>, Status> {
        handlers::issue_delete::delete_issue(request.into_inner()).await
    }

    async fn soft_delete_issue(
        &self,
        request: Request<SoftDeleteIssueRequest>,
    ) -> Result<Response<SoftDeleteIssueResponse>, Status> {
        handlers::issue_soft_delete::soft_delete_issue(request.into_inner()).await
    }

    async fn restore_issue(
        &self,
        request: Request<RestoreIssueRequest>,
    ) -> Result<Response<RestoreIssueResponse>, Status> {
        handlers::issue_restore::restore_issue(request.into_inner()).await
    }

    async fn move_issue(
        &self,
        request: Request<MoveIssueRequest>,
    ) -> Result<Response<MoveIssueResponse>, Status> {
        handlers::issue_move::move_issue(request.into_inner()).await
    }

    async fn duplicate_issue(
        &self,
        request: Request<DuplicateIssueRequest>,
    ) -> Result<Response<DuplicateIssueResponse>, Status> {
        handlers::issue_duplicate::duplicate_issue(request.into_inner()).await
    }

    async fn get_next_issue_number(
        &self,
        request: Request<GetNextIssueNumberRequest>,
    ) -> Result<Response<GetNextIssueNumberResponse>, Status> {
        handlers::issue_read::get_next_issue_number(request.into_inner()).await
    }

    async fn get_manifest(
        &self,
        request: Request<GetManifestRequest>,
    ) -> Result<Response<GetManifestResponse>, Status> {
        handlers::manifest::get_manifest(request.into_inner()).await
    }

    async fn get_config(
        &self,
        request: Request<GetConfigRequest>,
    ) -> Result<Response<GetConfigResponse>, Status> {
        handlers::config::get_config(request.into_inner()).await
    }

    async fn update_config(
        &self,
        request: Request<UpdateConfigRequest>,
    ) -> Result<Response<UpdateConfigResponse>, Status> {
        handlers::config_update::update_config(request.into_inner()).await
    }

    async fn is_initialized(
        &self,
        request: Request<IsInitializedRequest>,
    ) -> Result<Response<IsInitializedResponse>, Status> {
        handlers::init::is_initialized(request.into_inner()).await
    }

    async fn create_doc(
        &self,
        request: Request<CreateDocRequest>,
    ) -> Result<Response<CreateDocResponse>, Status> {
        handlers::doc_create::create_doc_handler(request.into_inner()).await
    }

    async fn get_doc(
        &self,
        request: Request<GetDocRequest>,
    ) -> Result<Response<GetDocResponse>, Status> {
        handlers::doc_read::get_doc_handler(request.into_inner()).await
    }

    async fn get_docs_by_slug(
        &self,
        request: Request<GetDocsBySlugRequest>,
    ) -> Result<Response<GetDocsBySlugResponse>, Status> {
        handlers::doc_read::get_docs_by_slug_handler(request.into_inner()).await
    }

    async fn list_docs(
        &self,
        request: Request<ListDocsRequest>,
    ) -> Result<Response<ListDocsResponse>, Status> {
        handlers::doc_read::list_docs_handler(request.into_inner()).await
    }

    async fn update_doc(
        &self,
        request: Request<UpdateDocRequest>,
    ) -> Result<Response<UpdateDocResponse>, Status> {
        handlers::doc_update::update_doc_handler(request.into_inner()).await
    }

    async fn delete_doc(
        &self,
        request: Request<DeleteDocRequest>,
    ) -> Result<Response<DeleteDocResponse>, Status> {
        handlers::doc_delete::delete_doc_handler(request.into_inner()).await
    }

    async fn soft_delete_doc(
        &self,
        request: Request<SoftDeleteDocRequest>,
    ) -> Result<Response<SoftDeleteDocResponse>, Status> {
        handlers::doc_soft_delete::soft_delete_doc_handler(request.into_inner()).await
    }

    async fn restore_doc(
        &self,
        request: Request<RestoreDocRequest>,
    ) -> Result<Response<RestoreDocResponse>, Status> {
        handlers::doc_restore::restore_doc_handler(request.into_inner()).await
    }

    async fn move_doc(
        &self,
        request: Request<MoveDocRequest>,
    ) -> Result<Response<MoveDocResponse>, Status> {
        handlers::doc_move::move_doc_handler(request.into_inner()).await
    }

    async fn duplicate_doc(
        &self,
        request: Request<DuplicateDocRequest>,
    ) -> Result<Response<DuplicateDocResponse>, Status> {
        handlers::doc_duplicate::duplicate_doc_handler(request.into_inner()).await
    }

    async fn add_asset(
        &self,
        request: Request<AddAssetRequest>,
    ) -> Result<Response<AddAssetResponse>, Status> {
        handlers::asset_add::add_asset(request.into_inner()).await
    }

    async fn list_assets(
        &self,
        request: Request<ListAssetsRequest>,
    ) -> Result<Response<ListAssetsResponse>, Status> {
        handlers::asset_read::list_assets(request.into_inner()).await
    }

    async fn get_asset(
        &self,
        request: Request<GetAssetRequest>,
    ) -> Result<Response<GetAssetResponse>, Status> {
        handlers::asset_read::get_asset(request.into_inner()).await
    }

    async fn delete_asset(
        &self,
        request: Request<DeleteAssetRequest>,
    ) -> Result<Response<DeleteAssetResponse>, Status> {
        handlers::asset_delete::delete_asset(request.into_inner()).await
    }

    async fn list_shared_assets(
        &self,
        request: Request<ListSharedAssetsRequest>,
    ) -> Result<Response<ListAssetsResponse>, Status> {
        handlers::asset_read::list_shared_assets(request.into_inner()).await
    }

    async fn list_projects(
        &self,
        request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        handlers::project::list_projects(request.into_inner()).await
    }

    async fn register_project(
        &self,
        request: Request<RegisterProjectRequest>,
    ) -> Result<Response<RegisterProjectResponse>, Status> {
        handlers::project_register::register_project(request.into_inner()).await
    }

    async fn untrack_project(
        &self,
        request: Request<UntrackProjectRequest>,
    ) -> Result<Response<UntrackProjectResponse>, Status> {
        handlers::project::untrack_project(request.into_inner()).await
    }

    async fn get_project_info(
        &self,
        request: Request<GetProjectInfoRequest>,
    ) -> Result<Response<GetProjectInfoResponse>, Status> {
        handlers::project::get_project_info(request.into_inner()).await
    }

    async fn set_project_favorite(
        &self,
        request: Request<SetProjectFavoriteRequest>,
    ) -> Result<Response<SetProjectFavoriteResponse>, Status> {
        handlers::project_settings::set_project_favorite(request.into_inner()).await
    }

    async fn set_project_archived(
        &self,
        request: Request<SetProjectArchivedRequest>,
    ) -> Result<Response<SetProjectArchivedResponse>, Status> {
        handlers::project_settings::set_project_archived(request.into_inner()).await
    }

    async fn set_project_organization(
        &self,
        request: Request<SetProjectOrganizationRequest>,
    ) -> Result<Response<SetProjectOrganizationResponse>, Status> {
        handlers::project_settings::set_project_organization(request.into_inner()).await
    }

    async fn set_project_user_title(
        &self,
        request: Request<SetProjectUserTitleRequest>,
    ) -> Result<Response<SetProjectUserTitleResponse>, Status> {
        handlers::project_settings::set_project_user_title(request.into_inner()).await
    }

    async fn set_project_title(
        &self,
        request: Request<SetProjectTitleRequest>,
    ) -> Result<Response<SetProjectTitleResponse>, Status> {
        handlers::project_title::set_project_title(request.into_inner()).await
    }

    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<CreateOrganizationResponse>, Status> {
        handlers::organization::create_organization(request.into_inner()).await
    }

    async fn list_organizations(
        &self,
        request: Request<ListOrganizationsRequest>,
    ) -> Result<Response<ListOrganizationsResponse>, Status> {
        handlers::organization::list_organizations(request.into_inner()).await
    }

    async fn get_organization(
        &self,
        request: Request<GetOrganizationRequest>,
    ) -> Result<Response<GetOrganizationResponse>, Status> {
        handlers::organization::get_organization(request.into_inner()).await
    }

    async fn update_organization(
        &self,
        request: Request<UpdateOrganizationRequest>,
    ) -> Result<Response<UpdateOrganizationResponse>, Status> {
        handlers::organization_write::update_organization(request.into_inner()).await
    }

    async fn delete_organization(
        &self,
        request: Request<DeleteOrganizationRequest>,
    ) -> Result<Response<DeleteOrganizationResponse>, Status> {
        handlers::organization_write::delete_organization(request.into_inner()).await
    }

    async fn get_daemon_info(
        &self,
        request: Request<GetDaemonInfoRequest>,
    ) -> Result<Response<DaemonInfo>, Status> {
        handlers::daemon::get_daemon_info(request.into_inner()).await
    }

    async fn shutdown(
        &self,
        request: Request<ShutdownRequest>,
    ) -> Result<Response<ShutdownResponse>, Status> {
        handlers::daemon::shutdown(request.into_inner(), &self.shutdown_tx).await
    }

    async fn restart(
        &self,
        request: Request<RestartRequest>,
    ) -> Result<Response<RestartResponse>, Status> {
        handlers::daemon_restart::restart(
            request.into_inner(),
            &self.shutdown_tx,
            self.exe_path.as_ref(),
        )
        .await
    }

    async fn create_link(
        &self,
        request: Request<CreateLinkRequest>,
    ) -> Result<Response<CreateLinkResponse>, Status> {
        handlers::link_create::create_link(request.into_inner()).await
    }

    async fn delete_link(
        &self,
        request: Request<DeleteLinkRequest>,
    ) -> Result<Response<DeleteLinkResponse>, Status> {
        handlers::link_delete::delete_link(request.into_inner()).await
    }

    async fn list_links(
        &self,
        request: Request<ListLinksRequest>,
    ) -> Result<Response<ListLinksResponse>, Status> {
        handlers::link_read::list_links(request.into_inner()).await
    }

    async fn get_available_link_types(
        &self,
        request: Request<GetAvailableLinkTypesRequest>,
    ) -> Result<Response<GetAvailableLinkTypesResponse>, Status> {
        handlers::link_read::get_available_link_types(request.into_inner()).await
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        handlers::user_create::create_user(request.into_inner()).await
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        handlers::user_read::get_user(request.into_inner()).await
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        handlers::user_read::list_users(request.into_inner()).await
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        handlers::user_update::update_user(request.into_inner()).await
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        handlers::user_delete::delete_user(request.into_inner()).await
    }

    async fn soft_delete_user(
        &self,
        request: Request<SoftDeleteUserRequest>,
    ) -> Result<Response<SoftDeleteUserResponse>, Status> {
        handlers::user_soft_delete::soft_delete_user(request.into_inner()).await
    }

    async fn restore_user(
        &self,
        request: Request<RestoreUserRequest>,
    ) -> Result<Response<RestoreUserResponse>, Status> {
        handlers::user_restore::restore_user(request.into_inner()).await
    }

    async fn sync_users(
        &self,
        request: Request<SyncUsersRequest>,
    ) -> Result<Response<SyncUsersResponse>, Status> {
        handlers::user_sync::sync_users(request.into_inner()).await
    }

    async fn advanced_search(
        &self,
        request: Request<AdvancedSearchRequest>,
    ) -> Result<Response<AdvancedSearchResponse>, Status> {
        handlers::search::advanced_search(request.into_inner()).await
    }

    async fn get_supported_editors(
        &self,
        request: Request<GetSupportedEditorsRequest>,
    ) -> Result<Response<GetSupportedEditorsResponse>, Status> {
        handlers::workspace_manage::get_supported_editors(request.into_inner()).await
    }

    async fn open_in_temp_workspace(
        &self,
        request: Request<OpenInTempWorkspaceWithEditorRequest>,
    ) -> Result<Response<OpenInTempWorkspaceResponse>, Status> {
        handlers::workspace_temp::open_in_temp_workspace(request.into_inner()).await
    }

    async fn open_in_temp_vscode(
        &self,
        request: Request<OpenInTempWorkspaceRequest>,
    ) -> Result<Response<OpenInTempWorkspaceResponse>, Status> {
        handlers::workspace_temp_vscode::open_in_temp_vscode(request.into_inner()).await
    }

    async fn open_in_temp_terminal(
        &self,
        request: Request<OpenInTempWorkspaceRequest>,
    ) -> Result<Response<OpenInTempWorkspaceResponse>, Status> {
        handlers::workspace_temp_terminal::open_in_temp_terminal(request.into_inner()).await
    }

    async fn open_agent_in_terminal(
        &self,
        request: Request<OpenAgentInTerminalRequest>,
    ) -> Result<Response<OpenAgentInTerminalResponse>, Status> {
        handlers::workspace_agent::open_agent_in_terminal(request.into_inner()).await
    }

    async fn list_temp_workspaces(
        &self,
        request: Request<ListTempWorkspacesRequest>,
    ) -> Result<Response<ListTempWorkspacesResponse>, Status> {
        handlers::workspace_manage::list_temp_workspaces(request.into_inner()).await
    }

    async fn close_temp_workspace(
        &self,
        request: Request<CloseTempWorkspaceRequest>,
    ) -> Result<Response<CloseTempWorkspaceResponse>, Status> {
        handlers::workspace_manage::close_temp_workspace(request.into_inner()).await
    }

    async fn cleanup_expired_workspaces(
        &self,
        request: Request<CleanupExpiredWorkspacesRequest>,
    ) -> Result<Response<CleanupExpiredWorkspacesResponse>, Status> {
        handlers::workspace_cleanup::cleanup_expired_workspaces(request.into_inner()).await
    }

    async fn open_standalone_workspace(
        &self,
        request: Request<OpenStandaloneWorkspaceWithEditorRequest>,
    ) -> Result<Response<OpenStandaloneWorkspaceResponse>, Status> {
        handlers::workspace_standalone::open_standalone_workspace(request.into_inner()).await
    }

    async fn open_standalone_workspace_vscode(
        &self,
        request: Request<OpenStandaloneWorkspaceRequest>,
    ) -> Result<Response<OpenStandaloneWorkspaceResponse>, Status> {
        handlers::workspace_standalone_vscode::open_standalone_workspace_vscode(
            request.into_inner(),
        )
        .await
    }

    async fn open_standalone_workspace_terminal(
        &self,
        request: Request<OpenStandaloneWorkspaceRequest>,
    ) -> Result<Response<OpenStandaloneWorkspaceResponse>, Status> {
        handlers::workspace_standalone_terminal::open_standalone_workspace_terminal(
            request.into_inner(),
        )
        .await
    }

    async fn get_entity_actions(
        &self,
        request: Request<GetEntityActionsRequest>,
    ) -> Result<Response<GetEntityActionsResponse>, Status> {
        handlers::entity_actions::get_entity_actions(request.into_inner()).await
    }

    async fn list_sync_conflicts(
        &self,
        request: Request<ListSyncConflictsRequest>,
    ) -> Result<Response<ListSyncConflictsResponse>, Status> {
        handlers::sync::list_sync_conflicts(request.into_inner()).await
    }

    async fn get_sync_conflict(
        &self,
        request: Request<GetSyncConflictRequest>,
    ) -> Result<Response<GetSyncConflictResponse>, Status> {
        handlers::sync::get_sync_conflict(request.into_inner()).await
    }

    async fn resolve_sync_conflict(
        &self,
        request: Request<ResolveSyncConflictRequest>,
    ) -> Result<Response<ResolveSyncConflictResponse>, Status> {
        handlers::sync::resolve_sync_conflict(request.into_inner()).await
    }

    async fn get_sync_status(
        &self,
        request: Request<GetSyncStatusRequest>,
    ) -> Result<Response<GetSyncStatusResponse>, Status> {
        handlers::sync::get_sync_status(request.into_inner()).await
    }

    async fn sync_pull(
        &self,
        request: Request<SyncPullRequest>,
    ) -> Result<Response<SyncPullResponse>, Status> {
        handlers::sync::sync_pull(request.into_inner()).await
    }

    async fn sync_push(
        &self,
        request: Request<SyncPushRequest>,
    ) -> Result<Response<SyncPushResponse>, Status> {
        handlers::sync::sync_push(request.into_inner()).await
    }
}
