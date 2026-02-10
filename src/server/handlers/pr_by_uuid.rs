// Domain function accessed via fully-qualified path to avoid name conflict with handler
use crate::registry::{list_projects, ListProjectsOptions};
use crate::server::convert_entity::pr_to_proto;
use crate::server::proto::{
    GetPrsByUuidRequest, GetPrsByUuidResponse, PrWithProject as ProtoPrWithProject,
};
use crate::utils::format_display_path;
use tonic::{Response, Status};

pub async fn get_prs_by_uuid(
    req: GetPrsByUuidRequest,
) -> Result<Response<GetPrsByUuidResponse>, Status> {
    // Get all initialized projects from registry
    let projects = match list_projects(ListProjectsOptions::default()).await {
        Ok(p) => p,
        Err(e) => {
            return Ok(Response::new(GetPrsByUuidResponse {
                success: false,
                error: format!("Failed to list projects: {e}"),
                prs: vec![],
                total_count: 0,
                errors: vec![],
            }))
        }
    };

    match crate::item::entities::pr::get_prs_by_uuid(&req.uuid, &projects).await {
        Ok(result) => {
            let prs_with_projects: Vec<ProtoPrWithProject> = result
                .prs
                .into_iter()
                .map(|pwp| {
                    // Use default priority_levels of 3 for global search
                    let priority_levels = 3;

                    ProtoPrWithProject {
                        pr: Some(pr_to_proto(&pwp.pr, priority_levels)),
                        display_path: format_display_path(&pwp.project_path),
                        project_path: pwp.project_path,
                        project_name: pwp.project_name,
                    }
                })
                .collect();

            let total_count = prs_with_projects.len() as i32;

            Ok(Response::new(GetPrsByUuidResponse {
                prs: prs_with_projects,
                total_count,
                errors: result.errors,
                success: true,
                error: String::new(),
            }))
        }
        Err(e) => Ok(Response::new(GetPrsByUuidResponse {
            success: false,
            error: e.to_string(),
            prs: vec![],
            total_count: 0,
            errors: vec![],
        })),
    }
}
