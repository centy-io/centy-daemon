use std::path::Path;

use crate::item::generic::storage::{generic_get, generic_get_by_display_number};
use crate::registry::track_project_async;
use crate::server::convert_entity::{generic_item_to_proto, user_to_generic_item_proto};
use crate::server::proto::{GetItemRequest, GetItemResponse};
use crate::server::structured_error::to_error_json;
use crate::user::get_user;
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

pub async fn get_item(req: GetItemRequest) -> Result<Response<GetItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Route user type to user-specific handler
    let lower = req.item_type.to_lowercase();
    if lower == "user" || lower == "users" {
        return match get_user(project_path, &req.item_id).await {
            Ok(user) => Ok(Response::new(GetItemResponse {
                success: true,
                error: String::new(),
                item: Some(user_to_generic_item_proto(&user)),
            })),
            Err(e) => Ok(Response::new(GetItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                item: None,
            })),
        };
    }

    let (item_type, config) = match resolve_item_type_config(project_path, &req.item_type).await {
        Ok(pair) => pair,
        Err(e) => {
            return Ok(Response::new(GetItemResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                item: None,
            }));
        }
    };

    // Dispatch: if display_number is specified and > 0, look up by display number
    let result = match req.display_number {
        Some(dn) if dn > 0 => {
            generic_get_by_display_number(project_path, &item_type, &config, dn).await
        }
        _ => generic_get(project_path, &item_type, &req.item_id).await,
    };

    match result {
        Ok(item) => Ok(Response::new(GetItemResponse {
            success: true,
            error: String::new(),
            item: Some(generic_item_to_proto(&item, &item_type)),
        })),
        Err(e) => Ok(Response::new(GetItemResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            item: None,
        })),
    }
}
