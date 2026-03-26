use crate::hooks::HookOperation;
use crate::link::DeleteLinkOptions;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::error_mapping::ToStructuredError;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{DeleteLinkRequest, DeleteLinkResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};

mod core;

fn err_resp(
    cwd: &str,
    e: &(impl std::fmt::Display + ToStructuredError),
) -> Response<DeleteLinkResponse> {
    Response::new(DeleteLinkResponse {
        success: false,
        error: to_error_json(cwd, e),
        ..Default::default()
    })
}

pub async fn delete_link(req: DeleteLinkRequest) -> Result<Response<DeleteLinkResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(err_resp(&req.project_path, &e));
    }
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.link_id.clone();
    let hook_request_data = serde_json::json!({ "link_id": &req.link_id });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        "link",
        HookOperation::Delete,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(err_resp(&req.project_path, &e));
    }
    let options = DeleteLinkOptions {
        link_id: req.link_id,
    };
    core::run_delete_link(
        project_path,
        options,
        hook_project_path,
        hook_item_id,
        hook_request_data,
        &req.project_path,
    )
    .await
}
