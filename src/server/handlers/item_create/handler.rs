use super::super::item_type_resolve::resolve_item_type_config;
use super::operation::{build_options, do_create};
use crate::hooks::HookOperation;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::helpers::{nonempty, nonzero_u32};
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{CreateItemRequest, CreateItemResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};
fn err_resp(
    cwd: &str,
    e: &(impl std::fmt::Display + crate::server::error_mapping::ToStructuredError),
) -> Response<CreateItemResponse> {
    Response::new(CreateItemResponse {
        success: false,
        error: to_error_json(cwd, e),
        ..Default::default()
    })
}
pub async fn create_item(req: CreateItemRequest) -> Result<Response<CreateItemResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(err_resp(&req.project_path, &e));
    }
    let (item_type, config) = match resolve_item_type_config(project_path, &req.item_type).await {
        Ok(pair) => pair,
        Err(e) => return Ok(err_resp(&req.project_path, &e)),
    };
    let hook_type = config.name.to_lowercase();
    let hook_project_path = req.project_path.clone();
    let hook_data = serde_json::json!({
        "item_type": &item_type, "title": &req.title,
        "body": &req.body, "priority": req.priority, "status": &req.status,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        &hook_type,
        HookOperation::Create,
        &hook_project_path,
        None,
        Some(hook_data.clone()),
    )
    .await
    {
        return Ok(err_resp(&req.project_path, &e));
    }
    let options = build_options(
        req.title,
        req.body,
        nonempty(req.status),
        nonzero_u32(req.priority),
        req.tags,
        &req.projects,
        req.custom_fields,
    );
    Ok(Response::new(
        do_create(
            project_path,
            &item_type,
            &config,
            &hook_type,
            &hook_project_path,
            hook_data,
            &req.project_path,
            options,
            req.org_wide,
        )
        .await,
    ))
}
