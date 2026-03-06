use crate::config::read_config;
use crate::hooks::HookOperation;
use crate::link::CreateLinkOptions;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::convert_link::proto_link_target_to_internal;
use crate::server::error_mapping::ToStructuredError;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{CreateLinkRequest, CreateLinkResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};

mod core;

fn err_resp(
    cwd: &str,
    e: &(impl std::fmt::Display + ToStructuredError),
) -> Response<CreateLinkResponse> {
    Response::new(CreateLinkResponse {
        success: false,
        error: to_error_json(cwd, e),
        ..Default::default()
    })
}

pub async fn create_link(req: CreateLinkRequest) -> Result<Response<CreateLinkResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(err_resp(&req.project_path, &e));
    }
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.source_id.clone();
    let hook_request_data = serde_json::json!({
        "source_id": &req.source_id, "target_id": &req.target_id, "link_type": &req.link_type,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        "link",
        HookOperation::Create,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(err_resp(&req.project_path, &e));
    }
    let source_type = proto_link_target_to_internal(req.source_type());
    let target_type = proto_link_target_to_internal(req.target_type());
    let custom_types = match read_config(project_path).await {
        Ok(Some(config)) => config.custom_link_types,
        Ok(None) | Err(_) => vec![],
    };
    let options = CreateLinkOptions {
        source_id: req.source_id,
        source_type,
        target_id: req.target_id,
        target_type,
        link_type: req.link_type,
    };
    core::run_create_link(
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
