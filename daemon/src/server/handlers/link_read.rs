use std::path::Path;

use crate::config::read_config;
use crate::link::TargetType;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::convert_link::link_view_to_proto;
use crate::server::proto::{
    GetAvailableLinkTypesRequest, GetAvailableLinkTypesResponse, LinkTypeInfo, ListLinksRequest,
    ListLinksResponse,
};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn list_links(req: ListLinksRequest) -> Result<Response<ListLinksResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path) {
        return Ok(Response::new(ListLinksResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    let entity_type = TargetType::new(req.entity_item_type.to_lowercase());

    match crate::link::list_links(project_path, &req.entity_id, entity_type).await {
        Ok(views) => Ok(Response::new(ListLinksResponse {
            links: views.iter().map(link_view_to_proto).collect(),
            total_count: views.len().try_into().unwrap_or(i32::MAX),
            success: true,
            error: String::new(),
        })),
        Err(e) => Ok(Response::new(ListLinksResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            links: vec![],
            total_count: 0,
        })),
    }
}

pub async fn get_available_link_types(
    req: GetAvailableLinkTypesRequest,
) -> Result<Response<GetAvailableLinkTypesResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    let custom_types = match read_config(project_path).await {
        Ok(Some(config)) => config.custom_link_types,
        Ok(None) | Err(_) => vec![],
    };

    let types = crate::link::get_available_link_types(&custom_types);

    Ok(Response::new(GetAvailableLinkTypesResponse {
        link_types: types
            .iter()
            .map(|t| LinkTypeInfo {
                name: t.name.clone(),
                description: t.description.clone().unwrap_or_default(),
                is_builtin: t.is_builtin,
            })
            .collect(),
    }))
}
