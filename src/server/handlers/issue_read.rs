use std::path::Path;

use crate::config::read_config;
use crate::registry::track_project_async;
use crate::server::convert_entity::issue_to_proto;
use crate::server::proto::{
    GetIssueByDisplayNumberRequest, GetIssueRequest, GetIssueResponse, GetNextIssueNumberRequest,
    GetNextIssueNumberResponse,
};
use crate::server::resolve::resolve_issue;
use crate::utils::get_centy_path;
use tonic::{Response, Status};

pub async fn get_issue(req: GetIssueRequest) -> Result<Response<GetIssueResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Read config for priority_levels (for label generation)
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

    match resolve_issue(project_path, &req.issue_id).await {
        Ok(issue) => Ok(Response::new(GetIssueResponse {
            success: true,
            error: String::new(),
            issue: Some(issue_to_proto(&issue, priority_levels)),
        })),
        Err(e) => Ok(Response::new(GetIssueResponse {
            success: false,
            error: e,
            issue: None,
        })),
    }
}

pub async fn get_issue_by_display_number(
    req: GetIssueByDisplayNumberRequest,
) -> Result<Response<GetIssueResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Read config for priority_levels (for label generation)
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

    match crate::item::entities::issue::get_issue_by_display_number(
        project_path,
        req.display_number,
    )
    .await
    {
        Ok(issue) => Ok(Response::new(GetIssueResponse {
            success: true,
            error: String::new(),
            issue: Some(issue_to_proto(&issue, priority_levels)),
        })),
        Err(e) => Ok(Response::new(GetIssueResponse {
            success: false,
            error: e.to_string(),
            issue: None,
        })),
    }
}

pub async fn get_next_issue_number(
    req: GetNextIssueNumberRequest,
) -> Result<Response<GetNextIssueNumberResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    let issues_path = get_centy_path(project_path).join("issues");

    #[allow(deprecated)]
    match crate::item::entities::issue::create::get_next_issue_number(&issues_path).await {
        Ok(issue_number) => Ok(Response::new(GetNextIssueNumberResponse { issue_number })),
        Err(e) => Err(Status::internal(e.to_string())),
    }
}
