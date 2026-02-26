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

#[allow(unknown_lints, max_lines_per_function)]
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

        let project_path = Path::new(&project.path);

        // Try to resolve the item type config for this project
        let (item_type, _config) =
            match resolve_item_type_config(project_path, &req.item_type).await {
                Ok(pair) => pair,
                Err(_) => continue, // Item type not configured in this project, skip
            };

        // Try to get the item by ID in this project
        match generic_get(project_path, &item_type, &req.item_id).await {
            Ok(item) => {
                let project_name = project.name.clone().unwrap_or_else(|| project.path.clone());

                found_items.push(ProtoItemWithProject {
                    item: Some(generic_item_to_proto(&item, &item_type)),
                    project_path: project.path.clone(),
                    project_name,
                    display_path: format_display_path(&project.path),
                });
            }
            Err(crate::item::core::error::ItemError::NotFound(_)) => {
                // Item not found in this project, skip
            }
            Err(crate::item::core::error::ItemError::NotInitialized) => {
                // Project not initialized, skip
            }
            Err(e) => {
                // Non-fatal error: record and continue
                errors.push(format!("{}: {e}", project.path));
            }
        }
    }

    let total_count = found_items.len() as i32;

    Ok(Response::new(SearchItemsResponse {
        items: found_items,
        total_count,
        errors,
        success: true,
        error: String::new(),
    }))
}
