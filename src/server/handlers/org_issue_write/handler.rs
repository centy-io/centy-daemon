#![allow(unknown_lints, max_lines_per_file)]
//! Handlers for org issue write RPCs.
use super::convert::org_issue_to_proto;
use crate::item::entities::issue::priority::default_priority;
use crate::registry::{
    create_org_issue, delete_org_issue, get_org_config, update_org_issue, UpdateOrgIssueOptions,
};
use crate::server::proto::{
    CreateOrgIssueRequest, CreateOrgIssueResponse, DeleteOrgIssueRequest, DeleteOrgIssueResponse,
    UpdateOrgIssueRequest, UpdateOrgIssueResponse,
};
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
pub async fn update_org_issue_handler(
    req: UpdateOrgIssueRequest,
) -> Result<Response<UpdateOrgIssueResponse>, Status> {
    let config = get_org_config(&req.organization_slug)
        .await
        .unwrap_or_default();
    let opts = UpdateOrgIssueOptions {
        title: if req.title.is_empty() {
            None
        } else {
            Some(req.title.clone())
        },
        description: if req.description.is_empty() {
            None
        } else {
            Some(req.description.clone())
        },
        status: if req.status.is_empty() {
            None
        } else {
            Some(req.status.clone())
        },
        priority: if req.priority == 0 {
            None
        } else {
            Some(req.priority as u32)
        },
        custom_fields: if req.custom_fields.is_empty() {
            None
        } else {
            Some(req.custom_fields)
        },
        add_referenced_projects: req.add_referenced_projects,
        remove_referenced_projects: req.remove_referenced_projects,
    };
    match update_org_issue(&req.organization_slug, &req.issue_id, opts).await {
        Ok(issue) => Ok(Response::new(UpdateOrgIssueResponse {
            success: true,
            error: String::new(),
            issue: Some(org_issue_to_proto(&issue, config.priority_levels)),
        })),
        Err(e) => Ok(Response::new(UpdateOrgIssueResponse {
            success: false,
            error: to_error_json(&req.issue_id, &e),
            issue: None,
        })),
    }
}
pub async fn delete_org_issue_handler(
    req: DeleteOrgIssueRequest,
) -> Result<Response<DeleteOrgIssueResponse>, Status> {
    match delete_org_issue(&req.organization_slug, &req.issue_id).await {
        Ok(()) => Ok(Response::new(DeleteOrgIssueResponse {
            success: true,
            error: String::new(),
        })),
        Err(e) => Ok(Response::new(DeleteOrgIssueResponse {
            success: false,
            error: to_error_json(&req.issue_id, &e),
        })),
    }
}
