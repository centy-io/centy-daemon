use std::path::Path;

use crate::item::core::crud::ItemFilters;
use crate::item::generic::storage::generic_list;
use crate::registry::track_project_async;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::proto::{ListItemsRequest, ListItemsResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

use super::item_type_resolve::{normalize_item_type, resolve_item_type_config};

pub async fn list_items(req: ListItemsRequest) -> Result<Response<ListItemsResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    let item_type = normalize_item_type(&req.item_type);

    let config = match resolve_item_type_config(project_path, &item_type).await {
        Ok(c) => c,
        Err(e) => {
            return Ok(Response::new(ListItemsResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                items: vec![],
                total_count: 0,
            }));
        }
    };

    let mut filters = ItemFilters::new();
    if !req.status.is_empty() {
        filters = filters.with_status(&req.status);
    }
    if req.priority != 0 {
        filters = filters.with_priority(req.priority as u32);
    }
    if req.include_deleted {
        filters = filters.include_deleted();
    }
    if req.limit > 0 {
        filters = filters.with_limit(req.limit as usize);
    }
    if req.offset > 0 {
        filters = filters.with_offset(req.offset as usize);
    }

    match generic_list(project_path, &config, filters).await {
        Ok(items) => {
            let total_count = items.len() as i32;
            Ok(Response::new(ListItemsResponse {
                success: true,
                error: String::new(),
                items: items.iter().map(generic_item_to_proto).collect(),
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
