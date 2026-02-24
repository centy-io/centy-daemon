use std::path::Path;

use crate::hooks::HookOperation;
use crate::registry::track_project_async;
use crate::server::convert_entity::user_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{SoftDeleteUserRequest, SoftDeleteUserResponse};
use crate::server::assert_service::assert_initialized;
use crate::server::structured_error::to_error_json;
use crate::user::soft_delete_user as internal_soft_delete_user;
use tonic::{Response, Status};

pub async fn soft_delete_user(
    req: SoftDeleteUserRequest,
) -> Result<Response<SoftDeleteUserResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(SoftDeleteUserResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.user_id.clone();
    let hook_request_data = serde_json::json!({
        "user_id": &req.user_id,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        "user",
        HookOperation::SoftDelete,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(SoftDeleteUserResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    match internal_soft_delete_user(project_path, &req.user_id).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                "user",
                HookOperation::SoftDelete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            Ok(Response::new(SoftDeleteUserResponse {
                success: true,
                error: String::new(),
                user: Some(user_to_proto(&result.user)),
                manifest: Some(manifest_to_proto(&result.manifest)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                "user",
                HookOperation::SoftDelete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(SoftDeleteUserResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                user: None,
                manifest: None,
            }))
        }
    }
}
