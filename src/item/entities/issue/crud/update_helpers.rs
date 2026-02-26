use super::super::metadata::IssueMetadata;
use super::super::planning::{add_planning_note, has_planning_note, is_planning_status};
use super::types::{Issue, IssueMetadataFlat};
use crate::utils::now_iso;
use std::collections::HashMap;

pub struct AppliedIssueUpdates {
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: u32,
    pub custom_fields: HashMap<String, String>,
    pub draft: bool,
}

pub fn build_updated_metadata(current: &Issue, updates: &AppliedIssueUpdates) -> IssueMetadata {
    IssueMetadata {
        common: mdstore::CommonMetadata {
            display_number: current.metadata.display_number,
            status: updates.status.clone(),
            priority: updates.priority,
            created_at: current.metadata.created_at.clone(),
            updated_at: now_iso(),
            custom_fields: updates
                .custom_fields
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        },
        draft: updates.draft,
        deleted_at: current.metadata.deleted_at.clone(),
        is_org_issue: current.metadata.is_org_issue,
        org_slug: current.metadata.org_slug.clone(),
        org_display_number: current.metadata.org_display_number,
    }
}

pub fn build_update_body(
    old_status: &str,
    new_status: &str,
    description: &str,
    current_content: &str,
) -> String {
    if is_planning_status(old_status) && is_planning_status(new_status) {
        if has_planning_note(current_content) {
            add_planning_note(description)
        } else {
            description.to_string()
        }
    } else if !is_planning_status(old_status) && is_planning_status(new_status) {
        add_planning_note(description)
    } else {
        description.to_string()
    }
}

pub fn build_issue_struct(
    issue_number: &str,
    updates: &AppliedIssueUpdates,
    current: &Issue,
    updated_at: &str,
) -> Issue {
    #[allow(deprecated)]
    Issue {
        id: issue_number.to_string(),
        issue_number: issue_number.to_string(),
        title: updates.title.clone(),
        description: updates.description.clone(),
        metadata: IssueMetadataFlat {
            display_number: current.metadata.display_number,
            status: updates.status.clone(),
            priority: updates.priority,
            created_at: current.metadata.created_at.clone(),
            updated_at: updated_at.to_string(),
            custom_fields: updates.custom_fields.clone(),
            draft: updates.draft,
            deleted_at: current.metadata.deleted_at.clone(),
            is_org_issue: current.metadata.is_org_issue,
            org_slug: current.metadata.org_slug.clone(),
            org_display_number: current.metadata.org_display_number,
        },
    }
}
