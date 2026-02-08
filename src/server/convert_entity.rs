use crate::item::entities::issue::priority_label;

use super::proto::{
    Doc, DocMetadata, Issue, IssueMetadata, PrMetadata, PullRequest, User as ProtoUser,
};

#[allow(deprecated)]
pub fn issue_to_proto(issue: &crate::item::entities::issue::Issue, priority_levels: u32) -> Issue {
    Issue {
        id: issue.id.clone(),
        display_number: issue.metadata.display_number,
        issue_number: issue.issue_number.clone(), // Legacy
        title: issue.title.clone(),
        description: issue.description.clone(),
        metadata: Some(IssueMetadata {
            display_number: issue.metadata.display_number,
            status: issue.metadata.status.clone(),
            priority: issue.metadata.priority as i32,
            created_at: issue.metadata.created_at.clone(),
            updated_at: issue.metadata.updated_at.clone(),
            custom_fields: issue.metadata.custom_fields.clone(),
            priority_label: priority_label(issue.metadata.priority, priority_levels),
            draft: issue.metadata.draft,
            deleted_at: issue.metadata.deleted_at.clone().unwrap_or_default(),
            is_org_issue: issue.metadata.is_org_issue,
            org_slug: issue.metadata.org_slug.clone().unwrap_or_default(),
            org_display_number: issue.metadata.org_display_number.unwrap_or(0),
        }),
    }
}

pub fn doc_to_proto(doc: &crate::item::entities::doc::Doc) -> Doc {
    Doc {
        slug: doc.slug.clone(),
        title: doc.title.clone(),
        content: doc.content.clone(),
        metadata: Some(DocMetadata {
            created_at: doc.metadata.created_at.clone(),
            updated_at: doc.metadata.updated_at.clone(),
            deleted_at: doc.metadata.deleted_at.clone().unwrap_or_default(),
            is_org_doc: doc.metadata.is_org_doc,
            org_slug: doc.metadata.org_slug.clone().unwrap_or_default(),
        }),
    }
}

pub fn pr_to_proto(
    pr: &crate::item::entities::pr::PullRequest,
    priority_levels: u32,
) -> PullRequest {
    PullRequest {
        id: pr.id.clone(),
        display_number: pr.metadata.display_number,
        title: pr.title.clone(),
        description: pr.description.clone(),
        metadata: Some(PrMetadata {
            display_number: pr.metadata.display_number,
            status: pr.metadata.status.clone(),
            source_branch: pr.metadata.source_branch.clone(),
            target_branch: pr.metadata.target_branch.clone(),
            reviewers: pr.metadata.reviewers.clone(),
            priority: pr.metadata.priority as i32,
            priority_label: priority_label(pr.metadata.priority, priority_levels),
            created_at: pr.metadata.created_at.clone(),
            updated_at: pr.metadata.updated_at.clone(),
            merged_at: pr.metadata.merged_at.clone(),
            closed_at: pr.metadata.closed_at.clone(),
            custom_fields: pr.metadata.custom_fields.clone(),
            deleted_at: pr.metadata.deleted_at.clone().unwrap_or_default(),
        }),
    }
}

pub fn user_to_proto(user: &crate::user::User) -> ProtoUser {
    ProtoUser {
        id: user.id.clone(),
        name: user.name.clone(),
        email: user.email.clone().unwrap_or_default(),
        git_usernames: user.git_usernames.clone(),
        created_at: user.created_at.clone(),
        updated_at: user.updated_at.clone(),
        deleted_at: user.deleted_at.clone().unwrap_or_default(),
    }
}
