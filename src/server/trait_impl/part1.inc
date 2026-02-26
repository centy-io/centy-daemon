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
