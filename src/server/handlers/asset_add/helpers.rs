use crate::hooks::HookOperation;
use crate::item::entities::issue::assets::AddAssetResult;
use crate::item::entities::issue::AssetError;
use crate::manifest::read_manifest;
use crate::server::convert_infra::{asset_info_to_proto, manifest_to_proto};
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{AddAssetRequest, AddAssetResponse};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::Response;

pub(super) struct AddAssetHookContext {
    pub(super) hook_project_path: String,
    pub(super) hook_item_id: String,
    pub(super) hook_request_data: serde_json::Value,
}

pub(super) async fn prepare_add_asset_hooks(
    req: &AddAssetRequest,
    project_path: &Path,
) -> Result<AddAssetHookContext, Response<AddAssetResponse>> {
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.filename.clone();
    let hook_request_data = serde_json::json!({
        "filename": &req.filename, "issue_id": &req.issue_id, "is_shared": req.is_shared,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        "asset",
        HookOperation::Create,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Err(Response::new(AddAssetResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }
    Ok(AddAssetHookContext {
        hook_project_path,
        hook_item_id,
        hook_request_data,
    })
}

pub(super) async fn finish_add_asset(
    result: Result<AddAssetResult, AssetError>,
    project_path: &Path,
    ctx: AddAssetHookContext,
    error_project_path: &str,
) -> Response<AddAssetResponse> {
    let success = result.is_ok();
    maybe_run_post_hooks(
        project_path,
        "asset",
        HookOperation::Create,
        &ctx.hook_project_path,
        Some(&ctx.hook_item_id),
        Some(ctx.hook_request_data),
        success,
    )
    .await;
    match result {
        Ok(result) => {
            let manifest = read_manifest(project_path).await.ok().flatten();
            Response::new(AddAssetResponse {
                success: true,
                error: String::new(),
                asset: Some(asset_info_to_proto(&result.asset)),
                path: result.path,
                manifest: manifest.map(|m| manifest_to_proto(&m)),
            })
        }
        Err(e) => Response::new(AddAssetResponse {
            success: false,
            error: to_error_json(error_project_path, &e),
            asset: None,
            path: String::new(),
            manifest: None,
        }),
    }
}
