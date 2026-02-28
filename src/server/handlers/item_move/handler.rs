use super::helpers::{
    assert_both_initialized, finish_move, prepare_move_hooks, resolve_configs, MoveHookContext,
};
use crate::item::generic::storage::{generic_move, generic_rename_slug};
use crate::registry::track_project_async;
use crate::server::proto::{MoveItemRequest, MoveItemResponse};
use mdstore::{IdStrategy, TypeConfig};
use std::path::Path;
use tonic::{Response, Status};
async fn execute_move(
    req: &MoveItemRequest,
    source_type: &str,
    target_type: &str,
    source_config: &TypeConfig,
    target_config: &TypeConfig,
    ctx: MoveHookContext,
) -> Result<Response<MoveItemResponse>, Status> {
    let source_path = Path::new(&req.source_project_path);
    let target_path = Path::new(&req.target_project_path);
    let is_slug_rename = req.source_project_path == req.target_project_path
        && !req.new_id.is_empty()
        && source_config.identifier == IdStrategy::Slug;
    let result = if is_slug_rename {
        generic_rename_slug(source_path, source_type, source_config, &req.item_id, &req.new_id)
            .await
    } else {
        let new_id = (!req.new_id.is_empty()).then_some(req.new_id.as_str());
        generic_move(
            source_path,
            target_path,
            source_type,
            target_type,
            source_config,
            target_config,
            &req.item_id,
            new_id,
        )
        .await
    };
    finish_move(
        result,
        source_path,
        target_path,
        target_type,
        &ctx.hook_type,
        &ctx.hook_project_path,
        &ctx.hook_item_id,
        ctx.hook_request_data,
        &req.source_project_path,
    )
    .await
}
pub async fn move_item(req: MoveItemRequest) -> Result<Response<MoveItemResponse>, Status> {
    track_project_async(req.source_project_path.clone());
    track_project_async(req.target_project_path.clone());
    let source_path = Path::new(&req.source_project_path);
    let target_path = Path::new(&req.target_project_path);
    if let Err(resp) = assert_both_initialized(&req, source_path, target_path).await {
        return Ok(resp);
    }
    let ((source_type, source_config), (target_type, target_config)) =
        match resolve_configs(&req, source_path, target_path).await {
            Ok(pair) => pair,
            Err(resp) => return Ok(resp),
        };
    let ctx = match prepare_move_hooks(&req, source_path, &source_config.name).await {
        Ok(ctx) => ctx,
        Err(resp) => return Ok(resp),
    };
    execute_move(&req, &source_type, &target_type, &source_config, &target_config, ctx).await
}
