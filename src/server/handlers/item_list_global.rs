use std::path::Path;

use crate::item::generic::storage::generic_list;
use crate::registry::{list_projects, ListProjectsOptions};
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::handlers::item_list::filters::build_filters_from_mql;
use crate::server::proto::{
    ItemWithProject as ProtoItemWithProject, ListItemsAcrossProjectsRequest,
    ListItemsAcrossProjectsResponse,
};
use crate::server::structured_error::StructuredError;
use crate::utils::format_display_path;
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

pub async fn list_items_across_projects(
    req: ListItemsAcrossProjectsRequest,
) -> Result<Response<ListItemsAcrossProjectsResponse>, Status> {
    let projects = match list_projects(ListProjectsOptions::default()).await {
        Ok(p) => p,
        Err(e) => {
            return Ok(Response::new(ListItemsAcrossProjectsResponse {
                success: false,
                error: StructuredError::new(
                    "",
                    "REGISTRY_ERROR",
                    format!("Failed to list projects: {e}"),
                )
                .to_json(),
                ..Default::default()
            }))
        }
    };

    let mut all_items: Vec<ProtoItemWithProject> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    // Query each project without per-project limit/offset — we paginate globally
    let per_project_filters = build_filters_from_mql(&req.filter, 0, 0);

    for project in &projects {
        if !project.initialized {
            continue;
        }
        collect_project_items(
            &project.path,
            project.name.as_deref(),
            &req.item_type,
            per_project_filters.clone(),
            &mut all_items,
            &mut errors,
        )
        .await;
    }

    // Sort all items by created_at descending
    all_items.sort_by(|a, b| {
        let ts_a = a
            .item
            .as_ref()
            .map_or("", |i| i.metadata.as_ref().map_or("", |m| &m.created_at));
        let ts_b = b
            .item
            .as_ref()
            .map_or("", |i| i.metadata.as_ref().map_or("", |m| &m.created_at));
        ts_b.cmp(ts_a)
    });

    let total_count = i32::try_from(all_items.len()).unwrap_or(i32::MAX);

    // Apply global pagination
    let offset = req.offset as usize;
    let items: Vec<ProtoItemWithProject> = if offset >= all_items.len() {
        vec![]
    } else if req.limit > 0 {
        all_items
            .into_iter()
            .skip(offset)
            .take(req.limit as usize)
            .collect()
    } else {
        all_items.into_iter().skip(offset).collect()
    };

    Ok(Response::new(ListItemsAcrossProjectsResponse {
        items,
        total_count,
        errors,
        success: true,
        error: String::new(),
    }))
}

async fn collect_project_items(
    project_path_str: &str,
    project_name_opt: Option<&str>,
    item_type_name: &str,
    filters: mdstore::Filters,
    all_items: &mut Vec<ProtoItemWithProject>,
    errors: &mut Vec<String>,
) {
    let project_path = Path::new(project_path_str);
    let Ok((item_type, _config)) = resolve_item_type_config(project_path, item_type_name).await
    else {
        return;
    };
    match generic_list(project_path, &item_type, filters).await {
        Ok(items) => {
            let project_name = project_name_opt.unwrap_or(project_path_str).to_owned();
            let display_path = format_display_path(project_path_str);
            for item in &items {
                all_items.push(ProtoItemWithProject {
                    item: Some(generic_item_to_proto(item, &item_type)),
                    project_path: project_path_str.to_owned(),
                    project_name: project_name.clone(),
                    display_path: display_path.clone(),
                });
            }
        }
        Err(crate::item::core::error::ItemError::NotInitialized) => {}
        Err(e) => {
            errors.push(format!("{project_path_str}: {e}"));
        }
    }
}
