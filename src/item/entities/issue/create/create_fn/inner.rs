use super::super::super::metadata::IssueFrontmatter;
use super::super::super::planning::{add_planning_note, is_planning_status};
use super::super::render::render_title_and_description;
use super::super::types::{CreateIssueOptions, CreateIssueResult, IssueError};
use super::super::write_issue::{build_frontmatter, persist_manifest, write_issue_file};
use crate::manifest::CentyManifest;
use std::collections::HashMap;
use std::path::Path;

pub async fn build_content(
    project_path: &Path,
    display_number: u32,
    status: &str,
    priority: u32,
    priority_levels: u32,
    now: &str,
    draft: bool,
    options: &CreateIssueOptions,
    custom_fields: HashMap<String, String>,
) -> Result<(IssueFrontmatter, String, String), IssueError> {
    let frontmatter = build_frontmatter(
        display_number,
        status,
        priority,
        now,
        draft,
        custom_fields,
    );
    let (display_title, description) = render_title_and_description(
        project_path,
        options,
        priority,
        priority_levels,
        status,
        &frontmatter,
    )
    .await?;
    let body = if is_planning_status(status) {
        add_planning_note(&description)
    } else {
        description
    };
    Ok((frontmatter, display_title, body))
}

pub async fn finalize_issue(
    issues_path: &Path,
    issue_id: String,
    frontmatter: &IssueFrontmatter,
    display_title: &str,
    body: &str,
    project_path: &Path,
    mut manifest: CentyManifest,
    display_number: u32,
) -> Result<CreateIssueResult, IssueError> {
    write_issue_file(issues_path, &issue_id, frontmatter, display_title, body).await?;
    persist_manifest(project_path, &mut manifest).await?;
    let created_files = vec![format!(".centy/issues/{issue_id}.md")];
    Ok(CreateIssueResult {
        id: issue_id,
        display_number,
        created_files,
        manifest,
    })
}
