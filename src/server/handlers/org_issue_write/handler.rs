//! Handlers for org issue write RPCs.
use crate::item::entities::issue::priority::default_priority;
use crate::registry::{create_org_issue, get_org_config};
use crate::server::proto::{CreateOrgIssueRequest, CreateOrgIssueResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn create_org_issue_handler(
    req: CreateOrgIssueRequest,
) -> Result<Response<CreateOrgIssueResponse>, Status> {
    let config = get_org_config(&req.organization_slug)
        .await
        .unwrap_or_default();
    let priority = if req.priority == 0 {
        default_priority(config.priority_levels)
    } else {
        req.priority as u32
    };
    let status = if req.status.is_empty() {
        "open".to_string()
    } else {
        req.status.clone()
    };
    match create_org_issue(
        &req.organization_slug,
        &req.title,
        &req.description,
        priority,
        &status,
        req.custom_fields,
        req.referenced_projects,
    )
    .await
    {
        Ok(issue) => Ok(Response::new(CreateOrgIssueResponse {
            success: true,
            error: String::new(),
            id: issue.id.clone(),
            display_number: issue.display_number,
            created_files: vec![format!(
                "~/.centy/orgs/{}/issues/{}.md",
                req.organization_slug, issue.id
            )],
        })),
        Err(e) => Ok(Response::new(CreateOrgIssueResponse {
            success: false,
            error: to_error_json(&req.organization_slug, &e),
            id: String::new(),
            display_number: 0,
            created_files: vec![],
        })),
    }
}
