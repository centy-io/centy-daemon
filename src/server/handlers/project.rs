use crate::registry::ListProjectsOptions;
use crate::server::convert_infra::project_info_to_proto;
use crate::server::proto::{
    GetProjectInfoRequest, GetProjectInfoResponse, ListProjectsRequest, ListProjectsResponse,
    UntrackProjectRequest, UntrackProjectResponse,
};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn list_projects(
    req: ListProjectsRequest,
) -> Result<Response<ListProjectsResponse>, Status> {
    let org_slug = if req.organization_slug.is_empty() {
        None
    } else {
        Some(req.organization_slug.as_str())
    };
    let opts = ListProjectsOptions {
        include_stale: req.include_stale,
        include_uninitialized: req.include_uninitialized,
        include_archived: req.include_archived,
        organization_slug: org_slug,
        ungrouped_only: req.ungrouped_only,
        include_temp: req.include_temp,
    };
    match crate::registry::list_projects(opts).await {
        Ok(projects) => {
            let total_count = projects.len() as i32;
            Ok(Response::new(ListProjectsResponse {
                projects: projects
                    .into_iter()
                    .map(|p| project_info_to_proto(&p))
                    .collect(),
                total_count,
                success: true,
                error: String::new(),
            }))
        }
        Err(e) => Ok(Response::new(ListProjectsResponse {
            success: false,
            error: to_error_json("", &e),
            projects: vec![],
            total_count: 0,
        })),
    }
}

pub async fn untrack_project(
    req: UntrackProjectRequest,
) -> Result<Response<UntrackProjectResponse>, Status> {
    match crate::registry::untrack_project(&req.project_path).await {
        Ok(()) => Ok(Response::new(UntrackProjectResponse {
            success: true,
            error: String::new(),
        })),
        Err(e) => Ok(Response::new(UntrackProjectResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
        })),
    }
}

pub async fn get_project_info(
    req: GetProjectInfoRequest,
) -> Result<Response<GetProjectInfoResponse>, Status> {
    match crate::registry::get_project_info(&req.project_path).await {
        Ok(Some(info)) => Ok(Response::new(GetProjectInfoResponse {
            found: true,
            project: Some(project_info_to_proto(&info)),
            success: true,
            error: String::new(),
        })),
        Ok(None) => Ok(Response::new(GetProjectInfoResponse {
            found: false,
            project: None,
            success: true,
            error: String::new(),
        })),
        Err(e) => Ok(Response::new(GetProjectInfoResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            found: false,
            project: None,
        })),
    }
}
