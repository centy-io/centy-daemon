use crate::hooks::HookOperation;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::error_mapping::ToStructuredError;
use crate::server::helpers::nonempty;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{UpdateUserRequest, UpdateUserResponse};
use crate::server::structured_error::to_error_json;
use crate::user::UpdateUserOptions;
use std::path::Path;
use tonic::{Response, Status};

mod core;

fn err_resp(
    cwd: &str,
    e: &(impl std::fmt::Display + ToStructuredError),
) -> Response<UpdateUserResponse> {
    Response::new(UpdateUserResponse {
        success: false,
        error: to_error_json(cwd, e),
        ..Default::default()
    })
}

pub async fn update_user(req: UpdateUserRequest) -> Result<Response<UpdateUserResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(err_resp(&req.project_path, &e));
    }
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.user_id.clone();
    let hook_request_data = serde_json::json!({
        "user_id": &req.user_id, "name": &req.name, "email": &req.email,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        "user",
        HookOperation::Update,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(err_resp(&req.project_path, &e));
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
    core::run_update_user(
        project_path,
        &req.user_id,
        options,
        hook_project_path,
        hook_item_id,
        hook_request_data,
        &req.project_path,
    )
    .await
}
