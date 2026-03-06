use super::super::super::metadata::IssueFrontmatter;
use super::super::super::planning::{add_planning_note, is_planning_status};
use super::super::helpers::build_issue_for_sync;
use super::super::render::render_title_and_description;
use super::super::types::{CreateIssueOptions, CreateIssueResult, IssueError};
use super::super::write_issue::{
    build_frontmatter, build_issue_metadata, persist_manifest, write_issue_file,
};
use crate::common::sync_to_org_projects;
use crate::manifest::CentyManifest;
use serde_json::Value;
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
    org_slug: Option<String>,
    org_display_number: Option<u32>,
    options: &CreateIssueOptions,
    custom_fields: HashMap<String, String>,
) -> Result<(IssueFrontmatter, String, String), IssueError> {
    let frontmatter = build_frontmatter(
        display_number,
        status,
        priority,
        now,
        draft,
        org_slug,
        org_display_number,
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
    org_slug: Option<String>,
    org_display_number: Option<u32>,
    status: String,
    priority: u32,
    custom_field_values: HashMap<String, Value>,
    draft: bool,
    options: &CreateIssueOptions,
) -> Result<CreateIssueResult, IssueError> {
    write_issue_file(issues_path, &issue_id, frontmatter, display_title, body).await?;
    persist_manifest(project_path, &mut manifest).await?;
    let created_files = vec![format!(".centy/issues/{issue_id}.md")];
    let metadata = build_issue_metadata(
        display_number,
        org_slug.as_deref(),
        org_display_number,
        status,
        priority,
        custom_field_values,
        draft,
    );
    let sync_results = if options.is_org_issue {
        let issue = build_issue_for_sync(&issue_id, options, display_number, &metadata);
        sync_to_org_projects(&issue, project_path).await
    } else {
        Vec::new()
    };
    Ok(CreateIssueResult {
        id: issue_id,
        display_number,
        org_display_number,
        created_files,
        manifest,
        sync_results,
    })
}
