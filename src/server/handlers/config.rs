use std::path::Path;

use crate::config::read_config;
use crate::registry::track_project_async;
use crate::server::config_to_proto::config_to_proto;
use crate::server::proto::{Config, GetConfigRequest, GetConfigResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn get_config(req: GetConfigRequest) -> Result<Response<GetConfigResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    match read_config(project_path).await {
        Ok(Some(config)) => Ok(Response::new(GetConfigResponse {
            success: true,
            error: String::new(),
            config: Some(config_to_proto(&config)),
        })),
        Ok(None) => Ok(Response::new(GetConfigResponse {
            success: true,
            error: String::new(),
            config: Some(Config {
                custom_fields: vec![],
                defaults: std::collections::HashMap::new(),
                priority_levels: 3, // Default
                allowed_states: vec![
                    "open".to_string(),
                    "in-progress".to_string(),
                    "closed".to_string(),
                ],
                default_state: "open".to_string(),
                version: crate::utils::CENTY_VERSION.to_string(),
                state_colors: std::collections::HashMap::new(),
                priority_colors: std::collections::HashMap::new(),
                custom_link_types: vec![],
                default_editor: String::new(),
                hooks: vec![],
                workspace: None,
            }),
        })),
        Err(e) => Ok(Response::new(GetConfigResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            config: None,
        })),
    }
}
