use super::super::item_type_resolve::resolve_item_type_config;
use super::filters::{build_filters_from_mql, parse_custom_field_filters};
use crate::item::generic::storage::generic_list;
use crate::registry::{get_org_projects, get_project_info, track_project_async};
use crate::server::assert_service::assert_initialized;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::proto::{GenericItem as ProtoGenericItem, ListItemsRequest, ListItemsResponse};
use crate::server::structured_error::to_error_json;
use mdstore::Filters;
use std::collections::HashMap;
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
        Ok(mut project_items) => {
            apply_custom_field_filters(&mut project_items, &custom_field_filters);
            let mut proto_items: Vec<ProtoGenericItem> = project_items
                .iter()
                .map(|item| generic_item_to_proto(item, &item_type))
                .collect();

            let include_org = req.include_organization_items.unwrap_or(true);
            if include_org {
                let org_filters = build_filters_from_mql(&req.filter, 0, 0);
                let org_proto_items = fetch_org_items(
                    &req.project_path,
                    &item_type,
                    org_filters,
                    &custom_field_filters,
                )
                .await;
                proto_items.extend(org_proto_items);
            }

            let total_count = proto_items.len().try_into().unwrap_or(i32::MAX);
            Ok(Response::new(ListItemsResponse {
                success: true,
                error: String::new(),
                items: proto_items,
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

fn apply_custom_field_filters(
    items: &mut Vec<mdstore::Item>,
    custom_field_filters: &HashMap<String, String>,
) {
    if !custom_field_filters.is_empty() {
        items.retain(|item| {
            custom_field_filters.iter().all(|(field, value)| {
                item.frontmatter
                    .custom_fields
                    .get(field)
                    .and_then(|v| v.as_str())
                    == Some(value.as_str())
            })
        });
    }
}

/// Fetch org-wide items for the given project and item type, filtered by the project's slug.
async fn fetch_org_items(
    project_path: &str,
    item_type: &str,
    filters: Filters,
    custom_field_filters: &HashMap<String, String>,
) -> Vec<ProtoGenericItem> {
    let Some(org_repo_path) = resolve_org_repo_path(project_path).await else {
        return vec![];
    };
    let project_slug = extract_project_slug(project_path);
    let org_data_root = Path::new(&org_repo_path);
    let org_type_dir = org_data_root.join(item_type);
    if !org_type_dir.exists() {
        return vec![];
    }
    let Ok(mut org_items) = mdstore::list(&org_type_dir, filters).await else {
        return vec![];
    };
    // Keep only items whose `projects` field contains the current project's slug
    if let Some(slug) = &project_slug {
        org_items.retain(|item| {
            item.frontmatter
                .custom_fields
                .get("projects")
                .and_then(|v| v.as_array())
                .is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some(slug.as_str())))
        });
    } else {
        return vec![];
    }
    apply_custom_field_filters(&mut org_items, custom_field_filters);
    org_items
        .iter()
        .map(|item| generic_item_to_proto(item, item_type))
        .collect()
}

/// Resolve the org repo path for a project.
/// The org repo is a tracked project in the same org whose path ends with "/.centy".
async fn resolve_org_repo_path(project_path: &str) -> Option<String> {
    let project_info = get_project_info(project_path).await.ok()??;
    let org_slug = project_info.organization_slug?;
    let org_projects = get_org_projects(&org_slug, Some(project_path)).await.ok()?;
    org_projects
        .into_iter()
        .find(|p| p.path.ends_with("/.centy"))
        .map(|p| p.path)
}

/// Extract the slug (folder name) from a project path.
fn extract_project_slug(project_path: &str) -> Option<String> {
    Path::new(project_path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
}

#[cfg(test)]
#[path = "../item_list_tests.rs"]
mod item_list_tests;
