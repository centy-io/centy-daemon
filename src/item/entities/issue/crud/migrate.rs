#![allow(unknown_lints, max_nesting_depth)]
use super::read::{read_issue_from_legacy_folder};
use super::types::{Issue, IssueCrudError};
use super::super::metadata::IssueFrontmatter;
use super::super::planning::{add_planning_note, has_planning_note, is_planning_status};
use crate::link::{read_links, write_links};
use crate::utils::format_markdown;
use mdstore::generate_frontmatter;
use std::path::Path;
use tokio::fs;
use tracing::debug;

#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines)]
pub async fn migrate_issue_to_new_format(
    issues_path: &Path,
    issue_folder_path: &Path,
    issue_id: &str,
) -> Result<Issue, IssueCrudError> {
    debug!("Auto-migrating issue {} to new format", issue_id);
    let issue = read_issue_from_legacy_folder(issue_folder_path, issue_id).await?;
    let frontmatter = IssueFrontmatter {
        display_number: issue.metadata.display_number,
        status: issue.metadata.status.clone(),
        priority: issue.metadata.priority,
        created_at: issue.metadata.created_at.clone(),
        updated_at: issue.metadata.updated_at.clone(),
        draft: issue.metadata.draft,
        deleted_at: issue.metadata.deleted_at.clone(),
        is_org_issue: issue.metadata.is_org_issue,
        org_slug: issue.metadata.org_slug.clone(),
        org_display_number: issue.metadata.org_display_number,
        custom_fields: issue.metadata.custom_fields.clone(),
    };
    let body = if is_planning_status(&issue.metadata.status)
        && !has_planning_note(&issue.description)
    {
        add_planning_note(&issue.description)
    } else {
        issue.description.clone()
    };
    let issue_content = generate_frontmatter(&frontmatter, &issue.title, &body);
    let new_issue_file = issues_path.join(format!("{issue_id}.md"));
    fs::write(&new_issue_file, format_markdown(&issue_content)).await?;
    let old_assets_path = issue_folder_path.join("assets");
    if old_assets_path.exists() && old_assets_path.is_dir() {
        let new_assets_path = issues_path.join("assets").join(issue_id);
        fs::create_dir_all(&new_assets_path).await?;
        let mut entries = fs::read_dir(&old_assets_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let file_name = entry.file_name();
            let old_file = entry.path();
            let new_file = new_assets_path.join(&file_name);
            fs::rename(&old_file, &new_file).await?;
        }
        debug!("Migrated assets for issue {}", issue_id);
    }
    let old_links = read_links(issue_folder_path).await?;
    if !old_links.links.is_empty() {
        write_links(issue_folder_path, &old_links).await?;
        debug!("Migrated links for issue {}", issue_id);
    }
    fs::remove_dir_all(issue_folder_path).await?;
    debug!("Deleted old issue folder for {}", issue_id);
    Ok(issue)
}
