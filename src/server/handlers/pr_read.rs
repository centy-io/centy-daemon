use std::path::Path;

use crate::config::read_config;
// Domain function accessed via fully-qualified path to avoid name conflict with handler
use crate::item::entities::pr::reconcile::get_next_pr_display_number;
use crate::registry::track_project_async;
use crate::server::convert_entity::pr_to_proto;
use crate::server::proto::{
    GetNextPrNumberRequest, GetNextPrNumberResponse, GetPrByDisplayNumberRequest, GetPrRequest,
    GetPrResponse,
};
use crate::server::resolve::resolve_pr;
use crate::server::structured_error::to_error_json;
use crate::utils::get_centy_path;
use tonic::{Response, Status};

pub async fn get_pr(req: GetPrRequest) -> Result<Response<GetPrResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Read config for priority_levels (for label generation)
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

    match resolve_pr(project_path, &req.pr_id).await {
        Ok(pr) => Ok(Response::new(GetPrResponse {
            success: true,
            error: String::new(),
            pr: Some(pr_to_proto(&pr, priority_levels)),
        })),
        Err(e) => Ok(Response::new(GetPrResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            pr: None,
        })),
    }
}

pub async fn get_pr_by_display_number(
    req: GetPrByDisplayNumberRequest,
) -> Result<Response<GetPrResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Read config for priority_levels (for label generation)
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

    match crate::item::entities::pr::get_pr_by_display_number(project_path, req.display_number)
        .await
    {
        Ok(pr) => Ok(Response::new(GetPrResponse {
            success: true,
            error: String::new(),
            pr: Some(pr_to_proto(&pr, priority_levels)),
        })),
        Err(e) => Ok(Response::new(GetPrResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            pr: None,
        })),
    }
}

pub async fn get_next_pr_number(
    req: GetNextPrNumberRequest,
) -> Result<Response<GetNextPrNumberResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    let prs_path = get_centy_path(project_path).join("prs");

    match get_next_pr_display_number(&prs_path).await {
        Ok(next_number) => Ok(Response::new(GetNextPrNumberResponse {
            next_number,
            success: true,
            error: String::new(),
        })),
        Err(e) => Ok(Response::new(GetNextPrNumberResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            next_number: 0,
        })),
    }
}
