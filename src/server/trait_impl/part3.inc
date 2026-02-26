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
