use std::path::Path;

use crate::config::read_config;
use crate::registry::track_project_async;
use crate::server::convert_link::{internal_link_to_proto, proto_link_target_to_internal};
use crate::server::proto::{
    GetAvailableLinkTypesRequest, GetAvailableLinkTypesResponse, LinkTypeInfo, ListLinksRequest,
    ListLinksResponse,
};
use tonic::{Response, Status};

pub async fn list_links(req: ListLinksRequest) -> Result<Response<ListLinksResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Convert proto type to internal type
    let entity_type = proto_link_target_to_internal(req.entity_type());

    match crate::link::list_links(project_path, &req.entity_id, entity_type).await {
        Ok(links_file) => Ok(Response::new(ListLinksResponse {
            links: links_file
                .links
                .iter()
                .map(internal_link_to_proto)
                .collect(),
            total_count: links_file.links.len() as i32,
        })),
        Err(e) => Err(Status::internal(e.to_string())),
    }
}

pub async fn get_available_link_types(
    req: GetAvailableLinkTypesRequest,
) -> Result<Response<GetAvailableLinkTypesResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Get custom link types from config
    let custom_types = match read_config(project_path).await {
        Ok(Some(config)) => config.custom_link_types,
        Ok(None) => vec![],
        Err(_) => vec![],
    };

    let types = crate::link::get_available_link_types(&custom_types);

    Ok(Response::new(GetAvailableLinkTypesResponse {
        link_types: types
            .iter()
            .map(|t| LinkTypeInfo {
                name: t.name.clone(),
                inverse: t.inverse.clone(),
                description: t.description.clone().unwrap_or_default(),
                is_builtin: t.is_builtin,
            })
            .collect(),
    }))
}
