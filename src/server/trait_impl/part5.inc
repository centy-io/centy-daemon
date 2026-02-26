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
