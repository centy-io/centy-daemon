use std::path::Path;

use crate::hooks::{HookItemType, HookOperation};
use crate::registry::track_project_async;
use crate::server::convert_entity::user_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{CreateUserRequest, CreateUserResponse};
use crate::server::structured_error::to_error_json;
use crate::user::{create_user as internal_create_user, CreateUserOptions};
use tonic::{Response, Status};

pub async fn create_user(req: CreateUserRequest) -> Result<Response<CreateUserResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_request_data = serde_json::json!({
        "id": &req.id,
        "name": &req.name,
        "email": &req.email,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::User,
        HookOperation::Create,
        &hook_project_path,
        None,
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(CreateUserResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    let options = CreateUserOptions {
        id: req.id,
        name: req.name,
        email: if req.email.is_empty() {
            None
        } else {
            Some(req.email)
        },
        git_usernames: req.git_usernames,
    };

    match internal_create_user(project_path, options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::User,
                HookOperation::Create,
                &hook_project_path,
                Some(&result.user.id),
                Some(hook_request_data),
                true,
            )
            .await;

            Ok(Response::new(CreateUserResponse {
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
                HookOperation::Create,
                &hook_project_path,
                None,
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(CreateUserResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                user: None,
                manifest: None,
            }))
        }
    }
}
