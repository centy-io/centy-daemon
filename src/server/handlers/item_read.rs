use std::path::Path;

use crate::item::generic::storage::generic_get;
use crate::registry::track_project_async;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::proto::{GetItemRequest, GetItemResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

pub async fn get_item(req: GetItemRequest) -> Result<Response<GetItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    let (item_type, _config) = match resolve_item_type_config(project_path, &req.item_type).await {
        Ok(pair) => pair,
        Err(e) => {
            return Ok(Response::new(GetItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                item: None,
            }));
        }
    };

    match generic_get(project_path, &item_type, &req.item_id).await {
        Ok(item) => Ok(Response::new(GetItemResponse {
            success: true,
            error: String::new(),
            item: Some(generic_item_to_proto(&item, &item_type)),
        })),
        Err(e) => Ok(Response::new(GetItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            item: None,
        })),
    }
}
