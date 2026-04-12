use super::super::metadata::IssueFrontmatter;
use super::super::planning::{add_planning_note, has_planning_note, is_planning_status};
use super::types::{Issue, IssueMetadataFlat};
use super::update_helpers::AppliedIssueUpdates;
use crate::utils::now_iso;

pub fn build_updated_frontmatter(
    current: &Issue,
    updates: &AppliedIssueUpdates,
) -> IssueFrontmatter {
    IssueFrontmatter {
        display_number: current.metadata.display_number,
        status: updates.status.clone(),
        priority: updates.priority,
        created_at: current.metadata.created_at.clone(),
        updated_at: now_iso(),
        draft: updates.draft,
        deleted_at: current.metadata.deleted_at.clone(),
        projects: current.metadata.projects.clone(),
        custom_fields: updates.custom_fields.clone(),
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
    Issue {
        id: issue_number.to_string(),
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
            projects: current.metadata.projects.clone(),
        },
    }
}
