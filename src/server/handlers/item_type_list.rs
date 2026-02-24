use std::path::Path;

use crate::config::item_type_config::ItemTypeRegistry;
use crate::registry::track_project_async;
use crate::server::convert_entity::config_to_proto;
use crate::server::proto::{ListItemTypesRequest, ListItemTypesResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn list_item_types(
    req: ListItemTypesRequest,
) -> Result<Response<ListItemTypesResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    match ItemTypeRegistry::build(project_path).await {
        Ok(registry) => {
            let item_types: Vec<_> = registry
                .all()
                .iter()
                .map(|(folder, config)| config_to_proto(folder, config))
                .collect();
            let total_count = item_types.len() as i32;
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
