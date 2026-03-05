use crate::hooks::HookOperation;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::error_mapping::ToStructuredError;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{DeleteAssetRequest, DeleteAssetResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};

mod core;

fn err_resp(
    cwd: &str,
    e: &(impl std::fmt::Display + ToStructuredError),
) -> Response<DeleteAssetResponse> {
    Response::new(DeleteAssetResponse {
        success: false,
        error: to_error_json(cwd, e),
        ..Default::default()
    })
}

pub async fn delete_asset(
    req: DeleteAssetRequest,
) -> Result<Response<DeleteAssetResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(err_resp(&req.project_path, &e));
    }
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.filename.clone();
    let hook_request_data = serde_json::json!({
        "filename": &req.filename, "issue_id": &req.issue_id, "is_shared": req.is_shared,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        "asset",
        HookOperation::Delete,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(err_resp(&req.project_path, &e));
    }
    let issue_id = (!req.issue_id.is_empty()).then_some(req.issue_id.as_str());
    core::run_delete_asset(
        project_path,
        issue_id,
        &req.filename,
        req.is_shared,
        hook_project_path,
        hook_item_id,
        hook_request_data,
        &req.project_path,
    )
    .await
}
