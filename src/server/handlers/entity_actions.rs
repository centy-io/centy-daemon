use std::path::Path;

use crate::config::item_type_config::read_item_type_config;
use crate::registry::track_project_async;
use crate::server::action_builders::build_issue_actions;
use crate::server::action_builders_extra::build_doc_actions;
use crate::server::proto::{EntityType, GetEntityActionsRequest, GetEntityActionsResponse};
use crate::server::resolve::resolve_issue;
use crate::server::structured_error::StructuredError;
use crate::workspace::terminal::is_terminal_available;
use crate::workspace::vscode::is_vscode_available;
use tonic::{Response, Status};

pub async fn get_entity_actions(
    req: GetEntityActionsRequest,
) -> Result<Response<GetEntityActionsResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    let has_entity_id = !req.entity_id.is_empty();

    let actions = match req.entity_type {
        t if t == EntityType::Issue as i32 => {
            let item_type_config = read_item_type_config(project_path, "issues")
                .await
                .ok()
                .flatten();
            let allowed_states = item_type_config
                .as_ref()
                .map(|c| c.statuses.clone())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    vec![
                        "open".to_string(),
                        "in-progress".to_string(),
                        "closed".to_string(),
                    ]
                });

            let entity_status = if has_entity_id {
                resolve_issue(project_path, &req.entity_id)
                    .await
                    .ok()
                    .map(|i| i.metadata.status)
            } else {
                None
            };
            build_issue_actions(
                entity_status.as_ref(),
                &allowed_states,
                is_vscode_available(),
                is_terminal_available(),
                has_entity_id,
            )
        }
        t if t == EntityType::Doc as i32 => build_doc_actions(has_entity_id),
        _ => {
            return Ok(Response::new(GetEntityActionsResponse {
                actions: vec![],
                success: false,
                error: StructuredError::new(
                    &req.project_path,
                    "UNKNOWN_ENTITY_TYPE",
                    "Unknown entity type".to_string(),
                )
                .to_json(),
            }))
        }
    };

    Ok(Response::new(GetEntityActionsResponse {
        actions,
        success: true,
        error: String::new(),
    }))
}
