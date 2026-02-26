use crate::item::entities::issue::priority::priority_label;
use crate::registry::OrgIssue;
use crate::server::proto::{OrgIssue as ProtoOrgIssue, OrgIssueMetadata};
pub(super) fn org_issue_to_proto(issue: &OrgIssue, priority_levels: u32) -> ProtoOrgIssue {
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
