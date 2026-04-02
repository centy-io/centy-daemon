use super::super::metadata::IssueFrontmatter;
use super::super::planning::{add_planning_note, has_planning_note, is_planning_status};
use super::read::read_issue_from_legacy_folder;
use super::types::{Issue, IssueCrudError};
use crate::utils::CENTY_HEADER_YAML;
use mdstore::generate_frontmatter;
use std::path::Path;
use tokio::fs;
use tracing::debug;

#[allow(clippy::cognitive_complexity)]
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
        custom_fields: issue.metadata.custom_fields.clone(),
    };
    let body =
        if is_planning_status(&issue.metadata.status) && !has_planning_note(&issue.description) {
            add_planning_note(&issue.description)
        } else {
            issue.description.clone()
        };
    let issue_content =
        generate_frontmatter(&frontmatter, &issue.title, &body, Some(CENTY_HEADER_YAML));
    let new_issue_file = issues_path.join(format!("{issue_id}.md"));
    fs::write(&new_issue_file, &issue_content).await?;
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
    fs::remove_dir_all(issue_folder_path).await?;
    debug!("Deleted old issue folder for {}", issue_id);
    Ok(issue)
}
