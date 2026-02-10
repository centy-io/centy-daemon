use std::path::Path;

use crate::hooks::{HookItemType, HookOperation};
use crate::item::entities::issue::AssetScope;
use crate::manifest::read_manifest;
use crate::registry::track_project_async;
use crate::server::convert_infra::{asset_info_to_proto, manifest_to_proto};
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{AddAssetRequest, AddAssetResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn add_asset(req: AddAssetRequest) -> Result<Response<AddAssetResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Pre-hook
    let hook_project_path = req.project_path.clone();
    let hook_item_id = req.filename.clone();
    let hook_request_data = serde_json::json!({
        "filename": &req.filename,
        "issue_id": &req.issue_id,
        "is_shared": req.is_shared,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::Asset,
        HookOperation::Create,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(AddAssetResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    let scope = if req.is_shared {
        AssetScope::Shared
    } else {
        AssetScope::IssueSpecific
    };

    let issue_id = if req.issue_id.is_empty() {
        None
    } else {
        Some(req.issue_id.as_str())
    };

    match crate::item::entities::issue::add_asset(
        project_path,
        issue_id,
        req.data,
        &req.filename,
        scope,
    )
    .await
    {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Asset,
                HookOperation::Create,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            // Re-read manifest for response
            let manifest = read_manifest(project_path).await.ok().flatten();
            Ok(Response::new(AddAssetResponse {
                success: true,
                error: String::new(),
                asset: Some(asset_info_to_proto(&result.asset)),
                path: result.path,
                manifest: manifest.map(|m| manifest_to_proto(&m)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Asset,
                HookOperation::Create,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(AddAssetResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                asset: None,
                path: String::new(),
                manifest: None,
            }))
        }
    }
}
