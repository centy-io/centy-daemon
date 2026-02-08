use std::path::Path;

use crate::manifest::read_manifest;
use crate::registry::track_project_async;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::proto::{GetManifestRequest, GetManifestResponse};
use tonic::{Response, Status};

pub async fn get_manifest(
    req: GetManifestRequest,
) -> Result<Response<GetManifestResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    match read_manifest(project_path).await {
        Ok(Some(manifest)) => Ok(Response::new(GetManifestResponse {
            success: true,
            error: String::new(),
            manifest: Some(manifest_to_proto(&manifest)),
        })),
        Ok(None) => Ok(Response::new(GetManifestResponse {
            success: false,
            error: "Manifest not found".to_string(),
            manifest: None,
        })),
        Err(e) => Ok(Response::new(GetManifestResponse {
            success: false,
            error: e.to_string(),
            manifest: None,
        })),
    }
}
