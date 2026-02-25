use std::path::Path;

use crate::hooks::HookOperation;
use crate::item::entities::issue::delete_asset as delete_asset_fn;
use crate::manifest::read_manifest;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{DeleteAssetRequest, DeleteAssetResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn delete_asset(
    req: DeleteAssetRequest,
) -> Result<Response<DeleteAssetResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(DeleteAssetResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

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
        "asset",
        HookOperation::Delete,
        &hook_project_path,
        Some(&hook_item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(DeleteAssetResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    let issue_id = if req.issue_id.is_empty() {
        None
    } else {
        Some(req.issue_id.as_str())
    };

    match delete_asset_fn(project_path, issue_id, &req.filename, req.is_shared).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                "asset",
                HookOperation::Delete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;

            // Re-read manifest for response
            let manifest = read_manifest(project_path).await.ok().flatten();
            Ok(Response::new(DeleteAssetResponse {
                success: true,
                error: String::new(),
                filename: result.filename,
                was_shared: result.was_shared,
                manifest: manifest.map(|m| manifest_to_proto(&m)),
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                "asset",
                HookOperation::Delete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;

            Ok(Response::new(DeleteAssetResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                filename: String::new(),
                was_shared: false,
                manifest: None,
            }))
        }
    }
}
