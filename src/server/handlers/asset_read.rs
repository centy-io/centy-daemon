use std::path::Path;

use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::convert_infra::asset_info_to_proto;
use crate::server::proto::{
    GetAssetRequest, GetAssetResponse, ListAssetsRequest, ListAssetsResponse,
    ListSharedAssetsRequest,
};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn list_assets(req: ListAssetsRequest) -> Result<Response<ListAssetsResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(ListAssetsResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    match crate::item::entities::issue::list_assets(project_path, &req.issue_id, req.include_shared)
        .await
    {
        Ok(assets) => {
            let total_count = assets.len() as i32;
            Ok(Response::new(ListAssetsResponse {
                assets: assets.iter().map(asset_info_to_proto).collect(),
                total_count,
                success: true,
                error: String::new(),
            }))
        }
        Err(e) => Ok(Response::new(ListAssetsResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            assets: vec![],
            total_count: 0,
        })),
    }
}

pub async fn get_asset(req: GetAssetRequest) -> Result<Response<GetAssetResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(GetAssetResponse {
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

    match crate::item::entities::issue::get_asset(
        project_path,
        issue_id,
        &req.filename,
        req.is_shared,
    )
    .await
    {
        Ok((data, asset_info)) => Ok(Response::new(GetAssetResponse {
            success: true,
            error: String::new(),
            data,
            asset: Some(asset_info_to_proto(&asset_info)),
        })),
        Err(e) => Ok(Response::new(GetAssetResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            data: vec![],
            asset: None,
        })),
    }
}

pub async fn list_shared_assets(
    req: ListSharedAssetsRequest,
) -> Result<Response<ListAssetsResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(ListAssetsResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    match crate::item::entities::issue::list_shared_assets(project_path).await {
        Ok(assets) => {
            let total_count = assets.len() as i32;
            Ok(Response::new(ListAssetsResponse {
                assets: assets.iter().map(asset_info_to_proto).collect(),
                total_count,
                success: true,
                error: String::new(),
            }))
        }
        Err(e) => Ok(Response::new(ListAssetsResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            assets: vec![],
            total_count: 0,
        })),
    }
}
