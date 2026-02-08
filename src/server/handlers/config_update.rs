use std::path::Path;

use crate::config::write_config;
use crate::registry::track_project_async;
use crate::server::config_to_proto::config_to_proto;
use crate::server::proto::{UpdateConfigRequest, UpdateConfigResponse};
use crate::server::proto_to_config::proto_to_config;
use crate::server::validate_config::validate_config;
use crate::utils::get_centy_path;
use tonic::{Response, Status};

pub async fn update_config(
    req: UpdateConfigRequest,
) -> Result<Response<UpdateConfigResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Check if project is initialized
    let centy_path = get_centy_path(project_path);
    let manifest_path = centy_path.join(".centy-manifest.json");
    if !manifest_path.exists() {
        return Ok(Response::new(UpdateConfigResponse {
            success: false,
            error: "Project not initialized".to_string(),
            config: None,
        }));
    }

    // Convert proto to internal config
    let proto_config = match req.config {
        Some(c) => c,
        None => {
            return Ok(Response::new(UpdateConfigResponse {
                success: false,
                error: "No config provided".to_string(),
                config: None,
            }));
        }
    };
    let config = proto_to_config(&proto_config);

    // Validate config
    if let Err(e) = validate_config(&config) {
        return Ok(Response::new(UpdateConfigResponse {
            success: false,
            error: e,
            config: None,
        }));
    }

    // Write config
    match write_config(project_path, &config).await {
        Ok(()) => Ok(Response::new(UpdateConfigResponse {
            success: true,
            error: String::new(),
            config: Some(config_to_proto(&config)),
        })),
        Err(e) => Ok(Response::new(UpdateConfigResponse {
            success: false,
            error: e.to_string(),
            config: None,
        })),
    }
}
