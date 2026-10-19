use std::path::Path;

use crate::config::item_type_config::{
    default_archived_config, default_comment_config, default_doc_config, default_epic_config,
    default_issue_config, default_user_story_config, ItemTypeConfig, ItemTypeRegistry,
};
use crate::config::read_config;
use crate::registry::track_project_async;
use crate::server::convert_entity::config_to_proto;
use crate::server::proto::{ListItemTypesRequest, ListItemTypesResponse};
use crate::server::structured_error::to_error_json;
use crate::utils::get_centy_path;
use tonic::{Response, Status};

/// Built-in defaults for the six types `init` creates. Same source as the
/// resolve fallback, so a type with a missing config.yaml still lists.
async fn builtin_item_types(project_path: &Path) -> Vec<(&'static str, ItemTypeConfig)> {
    let config = read_config(project_path)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    vec![
        ("issues", default_issue_config(&config)),
        ("docs", default_doc_config()),
        ("archived", default_archived_config()),
        ("comments", default_comment_config()),
        ("user_stories", default_user_story_config(&config)),
        ("epics", default_epic_config(&config)),
    ]
}

pub async fn list_item_types(
    req: ListItemTypesRequest,
) -> Result<Response<ListItemTypesResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    match ItemTypeRegistry::build(project_path).await {
        Ok(registry) => {
            // Only a folder that exists without config.yaml gets a default;
            // a folder the user removed stays unlisted.
            let centy_path = get_centy_path(project_path);
            let mut types = registry.all().clone();
            for (folder, config) in builtin_item_types(project_path).await {
                if centy_path.join(folder).is_dir() {
                    types.entry(folder.to_string()).or_insert(config);
                }
            }
            let item_types: Vec<_> = types
                .iter()
                .filter(|(_, config)| config.listed)
                .map(|(folder, config)| config_to_proto(folder, config))
                .collect();
            let total_count = item_types.len().try_into().unwrap_or(i32::MAX);
            Ok(Response::new(ListItemTypesResponse {
                success: true,
                error: String::new(),
                item_types,
                total_count,
            }))
        }
        Err(e) => Ok(Response::new(ListItemTypesResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            item_types: vec![],
            total_count: 0,
        })),
    }
}

#[cfg(test)]
#[path = "item_type_list_tests.rs"]
mod tests;
