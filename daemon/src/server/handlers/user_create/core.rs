use crate::hooks::HookOperation;
use crate::server::convert_entity::user_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::maybe_run_post_hooks;
use crate::server::proto::CreateUserResponse;
use crate::server::structured_error::to_error_json;
use crate::user::{create_user as internal_create_user, CreateUserOptions};
use std::path::Path;
use tonic::{Response, Status};

pub async fn run_create_user(
    project_path: &Path,
    options: CreateUserOptions,
    hook_project_path: String,
    hook_request_data: serde_json::Value,
    cwd: &str,
) -> Result<Response<CreateUserResponse>, Status> {
    match internal_create_user(project_path, options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                "user",
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
                "user",
                HookOperation::Create,
                &hook_project_path,
                None,
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(CreateUserResponse {
                success: false,
                error: to_error_json(cwd, &e),
                user: None,
                manifest: None,
            }))
        }
    }
}
