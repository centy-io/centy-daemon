use crate::registry::{list_projects, ListProjectsOptions};
use crate::server::convert_entity::issue_to_proto;
use crate::server::proto::{
    GetIssuesByUuidRequest, GetIssuesByUuidResponse, IssueWithProject as ProtoIssueWithProject,
};
use crate::utils::format_display_path;
use tonic::{Response, Status};

pub async fn get_issues_by_uuid(
    req: GetIssuesByUuidRequest,
) -> Result<Response<GetIssuesByUuidResponse>, Status> {
    // Get all initialized projects from registry
    let projects = match list_projects(ListProjectsOptions::default()).await {
        Ok(p) => p,
        Err(e) => return Err(Status::internal(format!("Failed to list projects: {e}"))),
    };

    match crate::item::entities::issue::get_issues_by_uuid(&req.uuid, &projects).await {
        Ok(result) => {
            let issues_with_projects: Vec<ProtoIssueWithProject> = result
                .issues
                .into_iter()
                .map(|iwp| {
                    // Use default priority_levels of 3 for global search
                    let priority_levels = 3;

                    ProtoIssueWithProject {
                        issue: Some(issue_to_proto(&iwp.issue, priority_levels)),
                        display_path: format_display_path(&iwp.project_path),
                        project_path: iwp.project_path,
                        project_name: iwp.project_name,
                    }
                })
                .collect();

            let total_count = issues_with_projects.len() as i32;

            Ok(Response::new(GetIssuesByUuidResponse {
                issues: issues_with_projects,
                total_count,
                errors: result.errors,
            }))
        }
        Err(e) => Err(Status::invalid_argument(e.to_string())),
    }
}
