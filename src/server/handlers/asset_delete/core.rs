use crate::hooks::HookOperation;
use crate::item::entities::issue::delete_asset as delete_asset_fn;
use crate::manifest::read_manifest;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::maybe_run_post_hooks;
use crate::server::proto::DeleteAssetResponse;
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};

pub async fn run_delete_asset(
    project_path: &Path,
    issue_id: Option<&str>,
    filename: &str,
    is_shared: bool,
    hook_project_path: String,
    hook_item_id: String,
    hook_request_data: serde_json::Value,
    cwd: &str,
) -> Result<Response<DeleteAssetResponse>, Status> {
    match delete_asset_fn(project_path, issue_id, filename, is_shared).await {
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
                error: to_error_json(cwd, &e),
                filename: String::new(),
                was_shared: false,
                manifest: None,
            }))
        }
    }
}
