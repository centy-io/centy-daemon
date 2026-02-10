use std::path::Path;

use crate::item::entities::doc::{get_doc, get_docs_by_slug, list_docs};
use crate::registry::{list_projects, track_project_async, ListProjectsOptions};
use crate::server::convert_entity::doc_to_proto;
use crate::server::proto::{
    DocWithProject as ProtoDocWithProject, GetDocRequest, GetDocResponse, GetDocsBySlugRequest,
    GetDocsBySlugResponse, ListDocsRequest, ListDocsResponse,
};
use crate::server::structured_error::{to_error_json, StructuredError};
use crate::utils::format_display_path;
use tonic::{Response, Status};

pub async fn get_doc_handler(req: GetDocRequest) -> Result<Response<GetDocResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    match get_doc(project_path, &req.slug).await {
        Ok(doc) => Ok(Response::new(GetDocResponse {
            success: true,
            error: String::new(),
            doc: Some(doc_to_proto(&doc)),
        })),
        Err(e) => Ok(Response::new(GetDocResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            doc: None,
        })),
    }
}

pub async fn get_docs_by_slug_handler(
    req: GetDocsBySlugRequest,
) -> Result<Response<GetDocsBySlugResponse>, Status> {
    // Get all initialized projects from registry
    let projects = match list_projects(ListProjectsOptions::default()).await {
        Ok(p) => p,
        Err(e) => {
            return Ok(Response::new(GetDocsBySlugResponse {
                success: false,
                error: StructuredError::new(
                    "",
                    "REGISTRY_ERROR",
                    format!("Failed to list projects: {e}"),
                )
                .to_json(),
                docs: vec![],
                total_count: 0,
                errors: vec![],
            }))
        }
    };

    match get_docs_by_slug(&req.slug, &projects).await {
        Ok(result) => {
            let docs_with_projects: Vec<ProtoDocWithProject> = result
                .docs
                .into_iter()
                .map(|dwp| ProtoDocWithProject {
                    doc: Some(doc_to_proto(&dwp.doc)),
                    display_path: format_display_path(&dwp.project_path),
                    project_path: dwp.project_path,
                    project_name: dwp.project_name,
                })
                .collect();

            let total_count = docs_with_projects.len() as i32;

            Ok(Response::new(GetDocsBySlugResponse {
                docs: docs_with_projects,
                total_count,
                errors: result.errors,
                success: true,
                error: String::new(),
            }))
        }
        Err(e) => Ok(Response::new(GetDocsBySlugResponse {
            success: false,
            error: to_error_json("", &e),
            docs: vec![],
            total_count: 0,
            errors: vec![],
        })),
    }
}

pub async fn list_docs_handler(req: ListDocsRequest) -> Result<Response<ListDocsResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    match list_docs(project_path, false).await {
        Ok(docs) => {
            let total_count = docs.len() as i32;
            Ok(Response::new(ListDocsResponse {
                docs: docs.into_iter().map(|d| doc_to_proto(&d)).collect(),
                total_count,
                success: true,
                error: String::new(),
            }))
        }
        Err(e) => Ok(Response::new(ListDocsResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            docs: vec![],
            total_count: 0,
        })),
    }
}
