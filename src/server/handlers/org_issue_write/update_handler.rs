use super::convert::org_issue_to_proto;
use crate::registry::{delete_org_issue, get_org_config, update_org_issue, UpdateOrgIssueOptions};
use crate::server::proto::{
    DeleteOrgIssueRequest, DeleteOrgIssueResponse, UpdateOrgIssueRequest, UpdateOrgIssueResponse,
};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

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
