use super::super::metadata::IssueFrontmatter;
use super::types::IssueError;
use crate::manifest::{update_manifest, write_manifest, CentyManifest};
use crate::utils::CENTY_HEADER_YAML;
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
        projects: vec![],
        custom_fields,
    }
}

pub async fn write_issue_file(
    issues_path: &Path,
    issue_id: &str,
    frontmatter: &IssueFrontmatter,
    display_title: &str,
    body: &str,
) -> Result<(), IssueError> {
    let issue_content =
        generate_frontmatter(frontmatter, display_title, body, Some(CENTY_HEADER_YAML));
    let issue_file = issues_path.join(format!("{issue_id}.md"));
    fs::write(&issue_file, &issue_content).await?;
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
