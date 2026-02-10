use crate::server::convert_infra::project_info_to_proto;
use crate::server::proto::{
    SetProjectArchivedRequest, SetProjectArchivedResponse, SetProjectFavoriteRequest,
    SetProjectFavoriteResponse, SetProjectOrganizationRequest, SetProjectOrganizationResponse,
    SetProjectUserTitleRequest, SetProjectUserTitleResponse,
};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn set_project_favorite(
    req: SetProjectFavoriteRequest,
) -> Result<Response<SetProjectFavoriteResponse>, Status> {
    match crate::registry::set_project_favorite(&req.project_path, req.is_favorite).await {
        Ok(info) => Ok(Response::new(SetProjectFavoriteResponse {
            success: true,
            error: String::new(),
            project: Some(project_info_to_proto(&info)),
        })),
        Err(e) => Ok(Response::new(SetProjectFavoriteResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            project: None,
        })),
    }
}

pub async fn set_project_archived(
    req: SetProjectArchivedRequest,
) -> Result<Response<SetProjectArchivedResponse>, Status> {
    match crate::registry::set_project_archived(&req.project_path, req.is_archived).await {
        Ok(info) => Ok(Response::new(SetProjectArchivedResponse {
            success: true,
            error: String::new(),
            project: Some(project_info_to_proto(&info)),
        })),
        Err(e) => Ok(Response::new(SetProjectArchivedResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            project: None,
        })),
    }
}

pub async fn set_project_organization(
    req: SetProjectOrganizationRequest,
) -> Result<Response<SetProjectOrganizationResponse>, Status> {
    let org_slug = if req.organization_slug.is_empty() {
        None
    } else {
        Some(req.organization_slug.as_str())
    };

    match crate::registry::set_project_organization(&req.project_path, org_slug).await {
        Ok(info) => Ok(Response::new(SetProjectOrganizationResponse {
            success: true,
            error: String::new(),
            project: Some(project_info_to_proto(&info)),
        })),
        Err(e) => Ok(Response::new(SetProjectOrganizationResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            project: None,
        })),
    }
}

pub async fn set_project_user_title(
    req: SetProjectUserTitleRequest,
) -> Result<Response<SetProjectUserTitleResponse>, Status> {
    let title = if req.title.is_empty() {
        None
    } else {
        Some(req.title)
    };

    match crate::registry::set_project_user_title(&req.project_path, title).await {
        Ok(info) => Ok(Response::new(SetProjectUserTitleResponse {
            success: true,
            error: String::new(),
            project: Some(project_info_to_proto(&info)),
        })),
        Err(e) => Ok(Response::new(SetProjectUserTitleResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            project: None,
        })),
    }
}
