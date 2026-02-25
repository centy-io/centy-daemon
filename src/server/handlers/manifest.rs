use std::path::Path;

use crate::manifest::read_manifest;
use crate::registry::track_project_async;
use crate::server::assert_service::assert_initialized;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::proto::{GetManifestRequest, GetManifestResponse};
use crate::server::structured_error::{to_error_json, StructuredError};
use tonic::{Response, Status};

pub async fn get_manifest(
    req: GetManifestRequest,
) -> Result<Response<GetManifestResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(GetManifestResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    match read_manifest(project_path).await {
        Ok(Some(manifest)) => Ok(Response::new(GetManifestResponse {
            success: true,
            error: String::new(),
            manifest: Some(manifest_to_proto(&manifest)),
        })),
        Ok(None) => Ok(Response::new(GetManifestResponse {
            success: false,
            error: StructuredError::new(
                &req.project_path,
                "MANIFEST_NOT_FOUND",
                "Manifest not found".to_string(),
            )
            .to_json(),
            manifest: None,
        })),
        Err(e) => Ok(Response::new(GetManifestResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            manifest: None,
        })),
    }
}
