use std::path::Path;

use crate::config::set_project_title as set_project_title_config;
use crate::registry::get_project_info;
use crate::server::convert_infra::project_info_to_proto;
use crate::server::proto::{SetProjectTitleRequest, SetProjectTitleResponse};
use tonic::{Response, Status};

pub async fn set_project_title(
    req: SetProjectTitleRequest,
) -> Result<Response<SetProjectTitleResponse>, Status> {
    let title = if req.title.is_empty() {
        None
    } else {
        Some(req.title)
    };
    let project_path = Path::new(&req.project_path);

    // Set project-scope title in .centy/project.json
    match set_project_title_config(project_path, title).await {
        Ok(()) => {
            // Fetch updated project info
            match get_project_info(&req.project_path).await {
                Ok(Some(info)) => Ok(Response::new(SetProjectTitleResponse {
                    success: true,
                    error: String::new(),
                    project: Some(project_info_to_proto(&info)),
                })),
                Ok(None) => Ok(Response::new(SetProjectTitleResponse {
                    success: false,
                    error: "Project not found in registry".to_string(),
                    project: None,
                })),
                Err(e) => Ok(Response::new(SetProjectTitleResponse {
                    success: false,
                    error: e.to_string(),
                    project: None,
                })),
            }
        }
        Err(e) => Ok(Response::new(SetProjectTitleResponse {
            success: false,
            error: e.to_string(),
            project: None,
        })),
    }
}
