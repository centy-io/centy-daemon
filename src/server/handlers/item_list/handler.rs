use super::super::item_type_resolve::resolve_item_type_config;
use super::filters::{build_filters_from_mql, parse_custom_field_filters};
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
    if let Err(e) = assert_initialized(project_path) {
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
    let custom_field_filters = parse_custom_field_filters(&req.filter);
    match generic_list(project_path, &item_type, filters).await {
        Ok(mut all_items) => {
            if !custom_field_filters.is_empty() {
                all_items.retain(|item| {
                    custom_field_filters.iter().all(|(field, value)| {
                        item.frontmatter
                            .custom_fields
                            .get(field)
                            .and_then(|v| v.as_str())
                            == Some(value.as_str())
                    })
                });
            }
            let total_count = all_items.len().try_into().unwrap_or(i32::MAX);
            Ok(Response::new(ListItemsResponse {
                success: true,
                error: String::new(),
                items: all_items
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
