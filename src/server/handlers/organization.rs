use crate::server::convert_infra::org_info_to_proto;
use crate::server::proto::{
    CreateOrganizationRequest, CreateOrganizationResponse, GetOrganizationRequest,
    GetOrganizationResponse, ListOrganizationsRequest, ListOrganizationsResponse,
};
use tonic::{Response, Status};

pub async fn create_organization(
    req: CreateOrganizationRequest,
) -> Result<Response<CreateOrganizationResponse>, Status> {
    let slug = if req.slug.is_empty() {
        None
    } else {
        Some(req.slug.as_str())
    };
    let description = if req.description.is_empty() {
        None
    } else {
        Some(req.description.as_str())
    };

    match crate::registry::create_organization(slug, &req.name, description).await {
        Ok(org) => Ok(Response::new(CreateOrganizationResponse {
            success: true,
            error: String::new(),
            organization: Some(org_info_to_proto(&org)),
        })),
        Err(e) => Ok(Response::new(CreateOrganizationResponse {
            success: false,
            error: e.to_string(),
            organization: None,
        })),
    }
}

pub async fn list_organizations(
    _req: ListOrganizationsRequest,
) -> Result<Response<ListOrganizationsResponse>, Status> {
    match crate::registry::list_organizations().await {
        Ok(orgs) => {
            let total_count = orgs.len() as i32;
            Ok(Response::new(ListOrganizationsResponse {
                organizations: orgs.into_iter().map(|o| org_info_to_proto(&o)).collect(),
                total_count,
            }))
        }
        Err(e) => Err(Status::internal(e.to_string())),
    }
}

pub async fn get_organization(
    req: GetOrganizationRequest,
) -> Result<Response<GetOrganizationResponse>, Status> {
    match crate::registry::get_organization(&req.slug).await {
        Ok(Some(org)) => Ok(Response::new(GetOrganizationResponse {
            found: true,
            organization: Some(org_info_to_proto(&org)),
        })),
        Ok(None) => Ok(Response::new(GetOrganizationResponse {
            found: false,
            organization: None,
        })),
        Err(e) => Err(Status::internal(e.to_string())),
    }
}
