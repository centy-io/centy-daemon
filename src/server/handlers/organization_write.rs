use crate::server::convert_infra::org_info_to_proto;
use crate::server::proto::{
    DeleteOrganizationRequest, DeleteOrganizationResponse, UpdateOrganizationRequest,
    UpdateOrganizationResponse,
};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn update_organization(
    req: UpdateOrganizationRequest,
) -> Result<Response<UpdateOrganizationResponse>, Status> {
    let name = if req.name.is_empty() {
        None
    } else {
        Some(req.name.as_str())
    };
    let description = if req.description.is_empty() {
        None
    } else {
        Some(req.description.as_str())
    };
    let new_slug = if req.new_slug.is_empty() {
        None
    } else {
        Some(req.new_slug.as_str())
    };

    match crate::registry::update_organization(&req.slug, name, description, new_slug).await {
        Ok(org) => Ok(Response::new(UpdateOrganizationResponse {
            success: true,
            error: String::new(),
            organization: Some(org_info_to_proto(&org)),
        })),
        Err(e) => Ok(Response::new(UpdateOrganizationResponse {
            success: false,
            error: to_error_json("", &e),
            organization: None,
        })),
    }
}

pub async fn delete_organization(
    req: DeleteOrganizationRequest,
) -> Result<Response<DeleteOrganizationResponse>, Status> {
    match crate::registry::delete_organization(&req.slug, req.cascade).await {
        Ok(unassigned_projects) => Ok(Response::new(DeleteOrganizationResponse {
            success: true,
            error: String::new(),
            unassigned_projects,
        })),
        Err(e) => Ok(Response::new(DeleteOrganizationResponse {
            success: false,
            error: to_error_json("", &e),
            unassigned_projects: 0,
        })),
    }
}
