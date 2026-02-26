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
