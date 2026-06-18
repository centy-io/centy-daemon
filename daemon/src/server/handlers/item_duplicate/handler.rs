use super::super::item_type_resolve::resolve_item_type_config;
use super::operation::{assert_both_initialized, do_duplicate, err_resp};
use crate::hooks::HookOperation;
use crate::item::generic::types::DuplicateGenericItemOptions;
use crate::registry::track_project_async;
use crate::server::helpers::nonempty;
use crate::server::hooks_helper::maybe_run_pre_hooks;
use crate::server::proto::{DuplicateItemRequest, DuplicateItemResponse};
use std::path::{Path, PathBuf};
use tonic::{Response, Status};
fn build_hook_data(req: &DuplicateItemRequest, item_type: &str) -> serde_json::Value {
    serde_json::json!({
        "item_type": item_type,
        "source_project_path": &req.source_project_path,
        "target_project_path": &req.target_project_path,
        "item_id": &req.item_id,
    })
}
pub async fn duplicate_item(
    req: DuplicateItemRequest,
) -> Result<Response<DuplicateItemResponse>, Status> {
    track_project_async(req.source_project_path.clone());
    track_project_async(req.target_project_path.clone());
    let source_path = Path::new(&req.source_project_path);
    let target_project_path = Path::new(&req.target_project_path);
    if let Err(resp) = assert_both_initialized(&req, source_path, target_project_path) {
        return Ok(resp);
    }
    let (item_type, config) =
        match resolve_item_type_config(target_project_path, &req.item_type).await {
            Ok(pair) => pair,
            Err(e) => return Ok(err_resp(&req.source_project_path, &e)),
        };
    let hook_type = config.name.to_lowercase();
    if !config.features.duplicate {
        let e = crate::item::core::error::ItemError::FeatureNotEnabled("duplicate".to_string());
        return Ok(err_resp(&req.source_project_path, &e));
    }
    let hook_project_path = req.source_project_path.clone();
    let hook_item_id = req.item_id.clone();
    let hook_request_data = build_hook_data(&req, &item_type);
    if let Err(e) = maybe_run_pre_hooks(
        Path::new(&hook_project_path),
        &hook_type,
        HookOperation::Duplicate,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(err_resp(&req.source_project_path, &e));
    }
    let options = DuplicateGenericItemOptions {
        source_project_path: PathBuf::from(&req.source_project_path),
        target_project_path: PathBuf::from(&req.target_project_path),
        item_id: req.item_id,
        new_id: nonempty(req.new_id),
        new_title: nonempty(req.new_title),
    };
    Ok(Response::new(
        do_duplicate(
            &item_type,
            &config,
            &hook_type,
            &hook_project_path,
            &hook_item_id,
            hook_request_data,
            &req.source_project_path,
            target_project_path,
            options,
        )
        .await,
    ))
}
