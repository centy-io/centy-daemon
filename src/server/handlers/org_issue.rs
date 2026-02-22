//! Handlers for org issue read RPCs.

use crate::item::entities::issue::priority::priority_label;
use crate::registry::{
    get_org_config, get_org_issue, get_org_issue_by_display_number, list_org_issues,
    ListOrgIssuesOptions, OrgIssue,
};
use crate::server::proto::{
    CustomFieldDefinition, GetOrgConfigRequest, GetOrgIssueByDisplayNumberRequest,
    GetOrgIssueRequest, ListOrgIssuesRequest, ListOrgIssuesResponse, OrgConfig as ProtoOrgConfig,
    OrgIssue as ProtoOrgIssue, OrgIssueMetadata,
};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

fn org_issue_to_proto(issue: &OrgIssue, priority_levels: u32) -> ProtoOrgIssue {
    ProtoOrgIssue {
        id: issue.id.clone(),
        display_number: issue.display_number,
        title: issue.title.clone(),
        description: issue.description.clone(),
        metadata: Some(OrgIssueMetadata {
            display_number: issue.display_number,
            status: issue.status.clone(),
            priority: issue.priority as i32,
            created_at: issue.created_at.clone(),
            updated_at: issue.updated_at.clone(),
            custom_fields: issue.custom_fields.clone(),
            priority_label: priority_label(issue.priority, priority_levels),
            referenced_projects: issue.referenced_projects.clone(),
        }),
    }
}

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
        priority: if req.priority == 0 {
            None
        } else {
            Some(req.priority as u32)
        },
        referenced_project: if req.referenced_project.is_empty() {
            None
        } else {
            Some(req.referenced_project.clone())
        },
    };

    match list_org_issues(&req.organization_slug, opts).await {
        Ok(issues) => {
            let total_count = issues.len() as i32;
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

pub async fn get_org_config_handler(
    req: GetOrgConfigRequest,
) -> Result<Response<ProtoOrgConfig>, Status> {
    match get_org_config(&req.organization_slug).await {
        Ok(config) => Ok(Response::new(ProtoOrgConfig {
            priority_levels: config.priority_levels,
            custom_fields: config
                .custom_fields
                .into_iter()
                .map(|f| CustomFieldDefinition {
                    name: f.name,
                    default_value: f.default_value.unwrap_or_default(),
                    field_type: "string".to_string(),
                    required: false,
                    enum_values: vec![],
                })
                .collect(),
        })),
        Err(e) => Err(Status::internal(to_error_json(&req.organization_slug, &e))),
    }
}
