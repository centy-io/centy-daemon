use std::path::Path;

use crate::config::write_config;
use crate::registry::track_project_async;
use crate::server::config_to_proto::config_to_proto;
use crate::server::proto::{UpdateConfigRequest, UpdateConfigResponse};
use crate::server::assert_service::assert_initialized;
use crate::server::proto_to_config::proto_to_config;
use crate::server::structured_error::{to_error_json, StructuredError};
use crate::server::validate_config::validate_config;
use tonic::{Response, Status};

pub async fn update_config(
    req: UpdateConfigRequest,
) -> Result<Response<UpdateConfigResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    if let Err(e) = assert_initialized(project_path).await {
        return Ok(Response::new(UpdateConfigResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    // Convert proto to internal config
    let proto_config = match req.config {
        Some(c) => c,
        None => {
            return Ok(Response::new(UpdateConfigResponse {
                success: false,
                error: StructuredError::new(
                    &req.project_path,
                    "INVALID_REQUEST",
                    "No config provided".to_string(),
                )
                .to_json(),
                config: None,
            }));
        }
    };
    let config = proto_to_config(&proto_config);

    // Validate config
    if let Err(e) = validate_config(&config) {
        return Ok(Response::new(UpdateConfigResponse {
            success: false,
            error: StructuredError::new(&req.project_path, "VALIDATION_ERROR", e).to_json(),
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
            error: to_error_json(&req.project_path, &e),
            config: None,
        })),
    }
}
