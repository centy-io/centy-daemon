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
        name = "grpc.archive_item",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn archive_item(
        &self,
        request: Request<ArchiveItemRequest>,
    ) -> Result<Response<ArchiveItemResponse>, Status> {
        let _timer = OperationTimer::new("archive_item");
        handlers::item_archive::archive_item(request.into_inner()).await
    }

    #[instrument(
        name = "grpc.unarchive_item",
        skip(self, request),
        fields(request_id = %generate_request_id())
    )]
    async fn unarchive_item(
        &self,
        request: Request<UnarchiveItemRequest>,
    ) -> Result<Response<UnarchiveItemResponse>, Status> {
        let _timer = OperationTimer::new("unarchive_item");
        handlers::item_unarchive::unarchive_item(request.into_inner()).await
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
