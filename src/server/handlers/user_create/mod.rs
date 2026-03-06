use crate::hooks::HookOperation;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::error_mapping::ToStructuredError;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{CreateUserRequest, CreateUserResponse};
use crate::server::structured_error::to_error_json;
use crate::user::CreateUserOptions;
use std::path::Path;
use tonic::{Response, Status};

mod core;

fn err_resp(
    cwd: &str,
    e: &(impl std::fmt::Display + ToStructuredError),
) -> Response<CreateUserResponse> {
    Response::new(CreateUserResponse {
        success: false,
        error: to_error_json(cwd, e),
        ..Default::default()
    })
}

pub async fn create_user(req: CreateUserRequest) -> Result<Response<CreateUserResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(err_resp(&req.project_path, &e));
    }
    let hook_project_path = req.project_path.clone();
    let hook_request_data = serde_json::json!({
        "id": &req.id,
        "name": &req.name,
        "email": &req.email,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        "user",
        HookOperation::Create,
        &hook_project_path,
        None,
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(err_resp(&req.project_path, &e));
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
    core::run_create_user(
        project_path,
        options,
        hook_project_path,
        hook_request_data,
        &req.project_path,
    )
    .await
}
