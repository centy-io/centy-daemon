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
