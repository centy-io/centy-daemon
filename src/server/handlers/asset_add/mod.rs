mod helpers;
use crate::item::entities::issue::AssetScope;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::proto::{AddAssetRequest, AddAssetResponse};
use crate::server::structured_error::to_error_json;
use helpers::{finish_add_asset, prepare_add_asset_hooks};
use std::path::Path;
use tonic::{Response, Status};

pub async fn add_asset(req: AddAssetRequest) -> Result<Response<AddAssetResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(AddAssetResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }
    let ctx = match prepare_add_asset_hooks(&req, project_path).await {
        Ok(ctx) => ctx,
        Err(resp) => return Ok(resp),
    };
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
    let result = crate::item::entities::issue::add_asset(
        project_path,
        issue_id,
        req.data,
        &req.filename,
        scope,
    )
    .await;
    Ok(finish_add_asset(result, project_path, ctx, &req.project_path).await)
}
