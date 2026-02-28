use super::super::id::generate_issue_id;
use super::super::planning::{add_planning_note, is_planning_status};
use super::super::reconcile::get_next_display_number;
use super::super::status::validate_status_for_project;
use super::helpers::{
    build_custom_fields, build_issue_for_sync, resolve_org_info, resolve_priority,
};
use super::render::render_title_and_description;
use super::types::{CreateIssueOptions, CreateIssueResult, IssueError};
use super::write_issue::{
    build_frontmatter, build_issue_metadata, persist_manifest, write_issue_file,
};
use crate::common::sync_to_org_projects;
use crate::config::item_type_config::read_item_type_config;
use crate::config::read_config;
use crate::manifest::read_manifest;
use crate::utils::{get_centy_path, now_iso};
use std::path::Path;
use tokio::fs;

pub async fn create_issue(
    project_path: &Path,
    options: CreateIssueOptions,
) -> Result<CreateIssueResult, IssueError> {
    if options.title.trim().is_empty() {
        return Err(IssueError::TitleRequired);
    }
    let manifest = read_manifest(project_path)
        .await?
        .ok_or(IssueError::NotInitialized)?;
    let issues_path = get_centy_path(project_path).join("issues");
    fs::create_dir_all(&issues_path).await?;
    let (org_slug, org_display_number) =
        resolve_org_info(project_path, options.is_org_issue).await?;
    let issue_id = generate_issue_id();
    let display_number = get_next_display_number(&issues_path).await?;
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);
    let priority = resolve_priority(options.priority, config.as_ref(), priority_levels)?;
    let item_type_config = read_item_type_config(project_path, "issues")
        .await
        .ok()
        .flatten();
    let status = options.status.clone().unwrap_or_else(|| {
        item_type_config
            .as_ref()
            .and_then(|c| c.default_status.clone())
            .unwrap_or_else(|| "open".to_string())
    });
    validate_status_for_project(project_path, "issues", &status).await?;
    let custom_field_values = build_custom_fields(config.as_ref(), &options.custom_fields);
    let draft = options.draft.unwrap_or(false);
    let now = now_iso();
    let frontmatter = build_frontmatter(
        display_number,
        &status,
        priority,
        &now,
        draft,
        org_slug.clone(),
        org_display_number,
        options.custom_fields.clone(),
    );
    let (display_title, description) = render_title_and_description(
        project_path,
        &options,
        priority,
        priority_levels,
        &status,
        &frontmatter,
    )
    .await?;
    let body = if is_planning_status(&status) {
        add_planning_note(&description)
    } else {
        description
    };
    write_issue_file(&issues_path, &issue_id, &frontmatter, &display_title, &body).await?;
    let mut manifest = manifest;
    persist_manifest(project_path, &mut manifest).await?;
    let created_files = vec![format!(".centy/issues/{issue_id}.md")];
    let metadata = build_issue_metadata(
        display_number,
        &org_slug,
        org_display_number,
        status,
        priority,
        custom_field_values,
        draft,
    );
    let sync_results = if options.is_org_issue {
        let issue = build_issue_for_sync(&issue_id, &options, display_number, &metadata);
        sync_to_org_projects(&issue, project_path).await
    } else {
        Vec::new()
    };
    #[allow(deprecated)]
    Ok(CreateIssueResult {
        id: issue_id.clone(),
        display_number,
        org_display_number,
        issue_number: issue_id,
        created_files,
        manifest,
        sync_results,
    })
}
