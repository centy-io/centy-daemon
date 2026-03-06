use crate::hooks::HookOperation;
use crate::server::convert_entity::user_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::maybe_run_post_hooks;
use crate::server::proto::UpdateUserResponse;
use crate::server::structured_error::to_error_json;
use crate::user::{update_user as internal_update_user, UpdateUserOptions};
use std::path::Path;
use tonic::{Response, Status};

pub async fn run_update_user(
    project_path: &Path,
    user_id: &str,
    options: UpdateUserOptions,
    hook_project_path: String,
    hook_item_id: String,
    hook_request_data: serde_json::Value,
    cwd: &str,
) -> Result<Response<UpdateUserResponse>, Status> {
    match internal_update_user(project_path, user_id, options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                "user",
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;
            Ok(Response::new(UpdateUserResponse {
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
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(UpdateUserResponse {
                success: false,
                error: to_error_json(cwd, &e),
                user: None,
                manifest: None,
            }))
        }
    }
}
