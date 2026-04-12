use crate::hooks::HookOperation;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::error_mapping::ToStructuredError;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{SoftDeleteUserRequest, SoftDeleteUserResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};

mod core;

fn err_resp(
    cwd: &str,
    e: &(impl std::fmt::Display + ToStructuredError),
) -> Response<SoftDeleteUserResponse> {
    Response::new(SoftDeleteUserResponse {
        success: false,
        error: to_error_json(cwd, e),
        ..Default::default()
    })
}

pub async fn soft_delete_user(
    req: SoftDeleteUserRequest,
) -> Result<Response<SoftDeleteUserResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(err_resp(&req.project_path, &e));
    }
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
        return Ok(err_resp(&req.project_path, &e));
    }
    core::run_soft_delete_user(
        project_path,
        &req.user_id,
        hook_project_path,
        hook_item_id,
        hook_request_data,
        &req.project_path,
    )
    .await
}
