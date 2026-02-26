use super::super::item_type_resolve::resolve_item_type_config;
use super::filters::build_filters_from_mql;
use crate::item::generic::storage::generic_list;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::proto::{ListItemsRequest, ListItemsResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};
pub async fn list_items(req: ListItemsRequest) -> Result<Response<ListItemsResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(ListItemsResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }
    let (item_type, _config) = match resolve_item_type_config(project_path, &req.item_type).await {
        Ok(pair) => pair,
        Err(e) => {
            return Ok(Response::new(ListItemsResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                items: vec![],
                total_count: 0,
            }))
        }
    };
    let filters = build_filters_from_mql(&req.filter, req.limit, req.offset);
    match generic_list(project_path, &item_type, filters).await {
        Ok(items) => {
            let total_count = items.len() as i32;
            Ok(Response::new(ListItemsResponse {
                success: true,
                error: String::new(),
                items: items
                    .iter()
                    .map(|item| generic_item_to_proto(item, &item_type))
                    .collect(),
                total_count,
            }))
        }
        Err(e) => Ok(Response::new(ListItemsResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            items: vec![],
            total_count: 0,
        })),
    }
}
#[cfg(test)]
#[path = "../item_list_tests.rs"]
mod item_list_tests;
