use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::convert_infra::asset_info_to_proto;
use crate::server::proto::{ListAssetsResponse, ListSharedAssetsRequest};
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};

pub async fn list_shared_assets(
    req: ListSharedAssetsRequest,
) -> Result<Response<ListAssetsResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(Response::new(ListAssetsResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }
    match crate::item::entities::issue::list_shared_assets(project_path).await {
        Ok(assets) => Ok(Response::new(ListAssetsResponse {
            assets: assets.iter().map(asset_info_to_proto).collect(),
            total_count: assets.len().try_into().unwrap_or(i32::MAX),
            success: true,
            error: String::new(),
        })),
        Err(e) => Ok(Response::new(ListAssetsResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            assets: vec![],
            total_count: 0,
        })),
    }
}
