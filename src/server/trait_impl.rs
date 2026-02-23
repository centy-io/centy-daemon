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

    async fn create_org_issue(
        &self,
        request: Request<CreateOrgIssueRequest>,
    ) -> Result<Response<CreateOrgIssueResponse>, Status> {
        handlers::org_issue_write::create_org_issue_handler(request.into_inner()).await
    }

    async fn get_org_issue(
        &self,
        request: Request<GetOrgIssueRequest>,
    ) -> Result<Response<OrgIssue>, Status> {
        handlers::org_issue::get_org_issue_handler(request.into_inner()).await
    }

    async fn get_org_issue_by_display_number(
        &self,
        request: Request<GetOrgIssueByDisplayNumberRequest>,
    ) -> Result<Response<OrgIssue>, Status> {
        handlers::org_issue::get_org_issue_by_display_number_handler(request.into_inner()).await
    }

    async fn list_org_issues(
        &self,
        request: Request<ListOrgIssuesRequest>,
    ) -> Result<Response<ListOrgIssuesResponse>, Status> {
        handlers::org_issue::list_org_issues_handler(request.into_inner()).await
    }

    async fn update_org_issue(
        &self,
        request: Request<UpdateOrgIssueRequest>,
    ) -> Result<Response<UpdateOrgIssueResponse>, Status> {
        handlers::org_issue_write::update_org_issue_handler(request.into_inner()).await
    }

    async fn delete_org_issue(
        &self,
        request: Request<DeleteOrgIssueRequest>,
    ) -> Result<Response<DeleteOrgIssueResponse>, Status> {
        handlers::org_issue_write::delete_org_issue_handler(request.into_inner()).await
    }

    async fn get_org_config(
        &self,
        request: Request<GetOrgConfigRequest>,
    ) -> Result<Response<OrgConfig>, Status> {
        handlers::org_issue::get_org_config_handler(request.into_inner()).await
    }

    async fn update_org_config(
        &self,
        request: Request<UpdateOrgConfigRequest>,
    ) -> Result<Response<UpdateOrgConfigResponse>, Status> {
        handlers::org_issue_write::update_org_config_handler(request.into_inner()).await
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
        let req = request.into_inner();
        handlers::workspace_temp::open_in_temp_workspace(OpenInTempWorkspaceWithEditorRequest {
            project_path: req.project_path,
            issue_id: req.issue_id,
            action: req.action,
            agent_name: req.agent_name,
            ttl_hours: req.ttl_hours,
            editor_id: "vscode".to_string(),
        })
        .await
    }

    async fn open_in_temp_terminal(
        &self,
        request: Request<OpenInTempWorkspaceRequest>,
    ) -> Result<Response<OpenInTempWorkspaceResponse>, Status> {
        let req = request.into_inner();
        handlers::workspace_temp::open_in_temp_workspace(OpenInTempWorkspaceWithEditorRequest {
            project_path: req.project_path,
            issue_id: req.issue_id,
            action: req.action,
            agent_name: req.agent_name,
            ttl_hours: req.ttl_hours,
            editor_id: "terminal".to_string(),
        })
        .await
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
        let req = request.into_inner();
        handlers::workspace_standalone::open_standalone_workspace(
            OpenStandaloneWorkspaceWithEditorRequest {
                project_path: req.project_path,
                name: req.name,
                description: req.description,
                ttl_hours: req.ttl_hours,
                agent_name: req.agent_name,
                editor_id: String::new(),
            },
        )
        .await
    }

    async fn open_standalone_workspace_terminal(
        &self,
        request: Request<OpenStandaloneWorkspaceRequest>,
    ) -> Result<Response<OpenStandaloneWorkspaceResponse>, Status> {
        let req = request.into_inner();
        handlers::workspace_standalone::open_standalone_workspace(
            OpenStandaloneWorkspaceWithEditorRequest {
                project_path: req.project_path,
                name: req.name,
                description: req.description,
                ttl_hours: req.ttl_hours,
                agent_name: req.agent_name,
                editor_id: String::new(),
            },
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

    #[instrument(
        name = "grpc.create_item_type",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn create_item_type(
        &self,
        request: Request<CreateItemTypeRequest>,
    ) -> Result<Response<CreateItemTypeResponse>, Status> {
        let _timer = OperationTimer::new("create_item_type");
        handlers::item_type_create::create_item_type(request.into_inner()).await
    }

    #[instrument(
        name = "grpc.list_item_types",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn list_item_types(
        &self,
        request: Request<ListItemTypesRequest>,
    ) -> Result<Response<ListItemTypesResponse>, Status> {
        let _timer = OperationTimer::new("list_item_types");
        handlers::item_type_list::list_item_types(request.into_inner()).await
    }

    #[instrument(
        name = "grpc.create_item",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn create_item(
        &self,
        request: Request<CreateItemRequest>,
    ) -> Result<Response<CreateItemResponse>, Status> {
        let _timer = OperationTimer::new("create_item");
        handlers::item_create::create_item(request.into_inner()).await
    }

    #[instrument(
        name = "grpc.get_item",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn get_item(
        &self,
        request: Request<GetItemRequest>,
    ) -> Result<Response<GetItemResponse>, Status> {
        let _timer = OperationTimer::new("get_item");
        handlers::item_read::get_item(request.into_inner()).await
    }

    #[instrument(
        name = "grpc.list_items",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn list_items(
        &self,
        request: Request<ListItemsRequest>,
    ) -> Result<Response<ListItemsResponse>, Status> {
        let _timer = OperationTimer::new("list_items");
        handlers::item_list::list_items(request.into_inner()).await
    }

    #[instrument(
        name = "grpc.update_item",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn update_item(
        &self,
        request: Request<UpdateItemRequest>,
    ) -> Result<Response<UpdateItemResponse>, Status> {
        let _timer = OperationTimer::new("update_item");
        handlers::item_update::update_item(request.into_inner()).await
    }

    #[instrument(
        name = "grpc.delete_item",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn delete_item(
        &self,
        request: Request<DeleteItemRequest>,
    ) -> Result<Response<DeleteItemResponse>, Status> {
        let _timer = OperationTimer::new("delete_item");
        handlers::item_delete::delete_item(request.into_inner()).await
    }

    #[instrument(
        name = "grpc.soft_delete_item",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn soft_delete_item(
        &self,
        request: Request<SoftDeleteItemRequest>,
    ) -> Result<Response<SoftDeleteItemResponse>, Status> {
        let _timer = OperationTimer::new("soft_delete_item");
        handlers::item_soft_delete::soft_delete_item(request.into_inner()).await
    }

    #[instrument(
        name = "grpc.restore_item",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn restore_item(
        &self,
        request: Request<RestoreItemRequest>,
    ) -> Result<Response<RestoreItemResponse>, Status> {
        let _timer = OperationTimer::new("restore_item");
        handlers::item_restore::restore_item(request.into_inner()).await
    }

    #[instrument(
        name = "grpc.duplicate_item",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn duplicate_item(
        &self,
        request: Request<DuplicateItemRequest>,
    ) -> Result<Response<DuplicateItemResponse>, Status> {
        let _timer = OperationTimer::new("duplicate_item");
        handlers::item_duplicate::duplicate_item(request.into_inner()).await
    }

    #[instrument(
        name = "grpc.move_item",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn move_item(
        &self,
        request: Request<MoveItemRequest>,
    ) -> Result<Response<MoveItemResponse>, Status> {
        let _timer = OperationTimer::new("move_item");
        handlers::item_move::move_item(request.into_inner()).await
    }

    #[instrument(
        name = "grpc.search_items",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn search_items(
        &self,
        request: Request<SearchItemsRequest>,
    ) -> Result<Response<SearchItemsResponse>, Status> {
        let _timer = OperationTimer::new("search_items");
        handlers::item_search::search_items(request.into_inner()).await
    }
}
