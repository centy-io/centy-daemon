use std::path::Path;

use crate::hooks::{HookItemType, HookOperation};
use crate::registry::track_project_async;
use crate::server::convert_entity::user_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::helpers::nonempty;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{UpdateUserRequest, UpdateUserResponse};
use crate::user::{update_user as internal_update_user, UpdateUserOptions};
use tonic::{Response, Status};

pub async fn update_user(req: UpdateUserRequest) -> Result<Response<UpdateUserResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.user_id.clone();
    let hook_request_data = serde_json::json!({
        "user_id": &req.user_id,
        "name": &req.name,
        "email": &req.email,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::User,
        HookOperation::Update,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(UpdateUserResponse {
            success: false,
            error: e,
            ..Default::default()
        }));
    }

    let options = UpdateUserOptions {
        name: nonempty(req.name),
        email: nonempty(req.email),
        git_usernames: if req.git_usernames.is_empty() {
            None
        } else {
            Some(req.git_usernames)
        },
    };

    match internal_update_user(project_path, &req.user_id, options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::User,
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
                HookItemType::User,
                HookOperation::Update,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(UpdateUserResponse {
                success: false,
                error: e.to_string(),
                user: None,
                manifest: None,
            }))
        }
    }
}
