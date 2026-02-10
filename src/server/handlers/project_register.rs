use std::path::Path;

use crate::registry::{get_project_info, infer_organization_from_remote, set_project_organization};
use crate::server::convert_infra::{org_inference_to_proto, project_info_to_proto};
use crate::server::proto::{RegisterProjectRequest, RegisterProjectResponse};
use crate::server::structured_error::{to_error_json, StructuredError};
use tonic::{Response, Status};

pub async fn register_project(
    req: RegisterProjectRequest,
) -> Result<Response<RegisterProjectResponse>, Status> {
    let project_path = Path::new(&req.project_path);

    // Track the project (this creates or updates the entry)
    if let Err(e) = crate::registry::track_project(&req.project_path).await {
        return Ok(Response::new(RegisterProjectResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            project: None,
            org_inference: None,
        }));
    }

    // Infer organization from git remote
    let existing_org = get_project_info(&req.project_path)
        .await
        .ok()
        .flatten()
        .and_then(|info| info.organization_slug);
    let inference = infer_organization_from_remote(project_path, existing_org.as_deref()).await;

    // Auto-assign if no existing org and inference succeeded without mismatch
    if existing_org.is_none() && !inference.has_mismatch {
        if let Some(ref slug) = inference.inferred_org_slug {
            let _ = set_project_organization(&req.project_path, Some(slug)).await;
        }
    }

    // Get the project info (refresh after potential org assignment)
    match get_project_info(&req.project_path).await {
        Ok(Some(info)) => Ok(Response::new(RegisterProjectResponse {
            success: true,
            error: String::new(),
            project: Some(project_info_to_proto(&info)),
            org_inference: Some(org_inference_to_proto(&inference)),
        })),
        Ok(None) => Ok(Response::new(RegisterProjectResponse {
            success: false,
            error: StructuredError::new(
                &req.project_path,
                "REGISTRATION_ERROR",
                "Failed to retrieve project after registration".to_string(),
            )
            .to_json(),
            project: None,
            org_inference: Some(org_inference_to_proto(&inference)),
        })),
        Err(e) => Ok(Response::new(RegisterProjectResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            project: None,
            org_inference: Some(org_inference_to_proto(&inference)),
        })),
    }
}
