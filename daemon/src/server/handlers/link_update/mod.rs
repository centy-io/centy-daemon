use crate::config::read_config;
use crate::hooks::HookOperation;
use crate::link::UpdateLinkOptions;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::error_mapping::ToStructuredError;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{UpdateLinkRequest, UpdateLinkResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};

mod core;

fn err_resp(
    cwd: &str,
    e: &(impl std::fmt::Display + ToStructuredError),
) -> Response<UpdateLinkResponse> {
    Response::new(UpdateLinkResponse {
        success: false,
        error: to_error_json(cwd, e),
        ..Default::default()
    })
}

pub async fn update_link(req: UpdateLinkRequest) -> Result<Response<UpdateLinkResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(err_resp(&req.project_path, &e));
    }
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.link_id.clone();
    let hook_request_data = serde_json::json!({
        "link_id": &req.link_id, "link_type": &req.link_type,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        "link",
        HookOperation::Update,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(err_resp(&req.project_path, &e));
    }
    let custom_types = match read_config(project_path).await {
        Ok(Some(config)) => config.custom_link_types,
        Ok(None) | Err(_) => vec![],
    };
    let options = UpdateLinkOptions {
        link_id: req.link_id,
        link_type: req.link_type,
    };
    core::run_update_link(
        project_path,
        options,
        custom_types,
        hook_project_path,
        hook_item_id,
        hook_request_data,
        &req.project_path,
    )
    .await
}
