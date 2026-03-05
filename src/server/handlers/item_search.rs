use std::path::Path;

use crate::item::generic::storage::generic_get;
use crate::registry::{list_projects, ListProjectsOptions};
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::proto::{
    ItemWithProject as ProtoItemWithProject, SearchItemsRequest, SearchItemsResponse,
};
use crate::server::structured_error::StructuredError;
use crate::utils::format_display_path;
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

pub async fn search_items(
    req: SearchItemsRequest,
) -> Result<Response<SearchItemsResponse>, Status> {
    // Get all initialized projects from registry
    let projects = match list_projects(ListProjectsOptions::default()).await {
        Ok(p) => p,
        Err(e) => {
            return Ok(Response::new(SearchItemsResponse {
                success: false,
                error: StructuredError::new(
                    "",
                    "REGISTRY_ERROR",
                    format!("Failed to list projects: {e}"),
                )
                .to_json(),
                items: vec![],
                total_count: 0,
                errors: vec![],
            }))
        }
    };

    let mut found_items: Vec<ProtoItemWithProject> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for project in &projects {
        if !project.initialized {
            continue;
        }
        process_project(
            &project.path,
            project.name.as_deref(),
            &req.item_type,
            &req.item_id,
            &mut found_items,
            &mut errors,
        )
        .await;
    }

    let total_count = found_items.len().try_into().unwrap_or(i32::MAX);

    Ok(Response::new(SearchItemsResponse {
        items: found_items,
        total_count,
        errors,
        success: true,
        error: String::new(),
    }))
}

async fn process_project(
    project_path_str: &str,
    project_name_opt: Option<&str>,
    item_type_name: &str,
    item_id: &str,
    found_items: &mut Vec<ProtoItemWithProject>,
    errors: &mut Vec<String>,
) {
    let project_path = Path::new(project_path_str);
    let Ok((item_type, _config)) = resolve_item_type_config(project_path, item_type_name).await
    else {
        return;
    };
    match generic_get(project_path, &item_type, item_id).await {
        Ok(item) => {
            let project_name = project_name_opt.unwrap_or(project_path_str).to_owned();
            found_items.push(ProtoItemWithProject {
                item: Some(generic_item_to_proto(&item, &item_type)),
                project_path: project_path_str.to_owned(),
                project_name,
                display_path: format_display_path(project_path_str),
            });
        }
        Err(
            crate::item::core::error::ItemError::NotFound(_)
            | crate::item::core::error::ItemError::NotInitialized,
        ) => {}
        Err(e) => {
            errors.push(format!("{project_path_str}: {e}"));
        }
    }
}
