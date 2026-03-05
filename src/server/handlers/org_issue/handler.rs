//! Handlers for org issue read RPCs.
use super::convert::org_issue_to_proto;
use crate::registry::{
    get_org_config, get_org_issue, get_org_issue_by_display_number, list_org_issues,
    ListOrgIssuesOptions,
};
use crate::server::proto::{
    GetOrgIssueByDisplayNumberRequest, GetOrgIssueRequest, ListOrgIssuesRequest,
    ListOrgIssuesResponse, OrgIssue as ProtoOrgIssue,
};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};
pub async fn get_org_issue_handler(
    req: GetOrgIssueRequest,
) -> Result<Response<ProtoOrgIssue>, Status> {
    let config = get_org_config(&req.organization_slug)
        .await
        .unwrap_or_default();
    match get_org_issue(&req.organization_slug, &req.issue_id).await {
        Ok(issue) => Ok(Response::new(org_issue_to_proto(
            &issue,
            config.priority_levels,
        ))),
        Err(e) => Err(Status::not_found(to_error_json(&req.issue_id, &e))),
    }
}
pub async fn get_org_issue_by_display_number_handler(
    req: GetOrgIssueByDisplayNumberRequest,
) -> Result<Response<ProtoOrgIssue>, Status> {
    let config = get_org_config(&req.organization_slug)
        .await
        .unwrap_or_default();
    match get_org_issue_by_display_number(&req.organization_slug, req.display_number).await {
        Ok(issue) => Ok(Response::new(org_issue_to_proto(
            &issue,
            config.priority_levels,
        ))),
        Err(e) => Err(Status::not_found(to_error_json(
            &req.display_number.to_string(),
            &e,
        ))),
    }
}
pub async fn list_org_issues_handler(
    req: ListOrgIssuesRequest,
) -> Result<Response<ListOrgIssuesResponse>, Status> {
    let config = get_org_config(&req.organization_slug)
        .await
        .unwrap_or_default();
    let opts = ListOrgIssuesOptions {
        status: if req.status.is_empty() {
            None
        } else {
            Some(req.status.clone())
        },
        priority: u32::try_from(req.priority).ok().filter(|&p| p != 0),
        referenced_project: if req.referenced_project.is_empty() {
            None
        } else {
            Some(req.referenced_project.clone())
        },
    };
    match list_org_issues(&req.organization_slug, opts).await {
        Ok(issues) => {
            let total_count = i32::try_from(issues.len()).unwrap_or(i32::MAX);
            let proto_issues = issues
                .iter()
                .map(|i| org_issue_to_proto(i, config.priority_levels))
                .collect();
            Ok(Response::new(ListOrgIssuesResponse {
                issues: proto_issues,
                total_count,
            }))
        }
        Err(e) => Err(Status::internal(to_error_json(&req.organization_slug, &e))),
    }
}
