use super::super::metadata::{IssueFrontmatter, IssueMetadata};
use super::types::IssueError;
use crate::manifest::{update_manifest, write_manifest, CentyManifest};
use crate::utils::format_markdown;
use mdstore::generate_frontmatter;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

pub fn build_frontmatter(
    display_number: u32,
    status: &str,
    priority: u32,
    now: &str,
    draft: bool,
    org_slug: Option<String>,
    org_display_number: Option<u32>,
    custom_fields: HashMap<String, String>,
) -> IssueFrontmatter {
    IssueFrontmatter {
        display_number,
        status: status.to_string(),
        priority,
        created_at: now.to_string(),
        updated_at: now.to_string(),
        draft,
        deleted_at: None,
        is_org_issue: org_slug.is_some(),
        org_slug,
        org_display_number,
        custom_fields,
    }
}

#[allow(clippy::ref_option)]
pub fn build_issue_metadata(
    display_number: u32,
    org_slug: &Option<String>,
    org_display_number: Option<u32>,
    status: String,
    priority: u32,
    custom_field_values: HashMap<String, serde_json::Value>,
    draft: bool,
) -> IssueMetadata {
    if let Some(ref org) = org_slug {
        IssueMetadata::new_org_issue(
            display_number,
            org_display_number.unwrap_or(0),
            status,
            priority,
            org,
            custom_field_values,
            draft,
        )
    } else {
        IssueMetadata::new_draft(display_number, status, priority, custom_field_values, draft)
    }
}

pub async fn write_issue_file(
    issues_path: &Path,
    issue_id: &str,
    frontmatter: &IssueFrontmatter,
    display_title: &str,
    body: &str,
) -> Result<(), IssueError> {
    let issue_content = generate_frontmatter(frontmatter, display_title, body);
    let issue_file = issues_path.join(format!("{issue_id}.md"));
    fs::write(&issue_file, format_markdown(&issue_content)).await?;
    Ok(())
}

pub async fn persist_manifest(
    project_path: &Path,
    manifest: &mut CentyManifest,
) -> Result<(), IssueError> {
    update_manifest(manifest);
    write_manifest(project_path, manifest).await?;
    Ok(())
}
