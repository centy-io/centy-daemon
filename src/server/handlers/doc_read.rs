use std::path::Path;

use crate::item::entities::doc::{get_doc, get_docs_by_slug, list_docs};
use crate::registry::{list_projects, track_project_async, ListProjectsOptions};
use crate::server::convert_entity::doc_to_proto;
use crate::server::proto::{
    DocWithProject as ProtoDocWithProject, GetDocRequest, GetDocResponse, GetDocsBySlugRequest,
    GetDocsBySlugResponse, ListDocsRequest, ListDocsResponse,
};
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
            error: e.to_string(),
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
        Err(e) => return Err(Status::internal(format!("Failed to list projects: {e}"))),
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
            }))
        }
        Err(e) => Err(Status::invalid_argument(e.to_string())),
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
            }))
        }
        Err(e) => Err(Status::internal(e.to_string())),
    }
}
