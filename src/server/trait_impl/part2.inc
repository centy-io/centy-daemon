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
