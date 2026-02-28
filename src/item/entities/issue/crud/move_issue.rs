#![allow(unknown_lints, max_nesting_depth, max_lines_per_file)]
use super::super::assets::copy_assets_folder;
use super::super::metadata::IssueFrontmatter;
use super::super::reconcile::get_next_display_number;
use super::super::status::validate_status_for_project;
use super::read::{read_issue_from_frontmatter, read_issue_from_legacy_folder};
use super::types::{IssueCrudError, MoveIssueOptions, MoveIssueResult};
use crate::config::read_config;
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::{format_issue_file, get_centy_path, now_iso};
use mdstore::generate_frontmatter;
use tokio::fs;

pub async fn move_issue(options: MoveIssueOptions) -> Result<MoveIssueResult, IssueCrudError> {
    if options.source_project_path == options.target_project_path {
        return Err(IssueCrudError::SameProject);
    }
    let mut source_manifest = read_manifest(&options.source_project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;
    let mut target_manifest = read_manifest(&options.target_project_path)
        .await?
        .ok_or(IssueCrudError::TargetNotInitialized)?;
    let source_centy = get_centy_path(&options.source_project_path);
    let source_issues_path = source_centy.join("issues");
    let source_file_path = source_issues_path.join(format!("{}.md", &options.issue_id));
    let source_folder_path = source_issues_path.join(&options.issue_id);
    let (source_is_new_format, source_issue) = if source_file_path.exists() {
        (
            true,
            read_issue_from_frontmatter(&source_file_path, &options.issue_id).await?,
        )
    } else if source_folder_path.exists() {
        (
            false,
            read_issue_from_legacy_folder(&source_folder_path, &options.issue_id).await?,
        )
    } else {
        return Err(IssueCrudError::IssueNotFound(options.issue_id.clone()));
    };
    let old_display_number = source_issue.metadata.display_number;
    let target_config = read_config(&options.target_project_path)
        .await
        .ok()
        .flatten();
    let target_priority_levels = target_config.as_ref().map_or(3, |c| c.priority_levels);
    if source_issue.metadata.priority > target_priority_levels {
        return Err(IssueCrudError::InvalidPriorityInTarget(
            source_issue.metadata.priority,
        ));
    }
    validate_status_for_project(
        &options.target_project_path,
        "issues",
        &source_issue.metadata.status,
    )
    .await?;
    let target_centy = get_centy_path(&options.target_project_path);
    let target_issues_path = target_centy.join("issues");
    fs::create_dir_all(&target_issues_path).await?;
    let new_display_number = get_next_display_number(&target_issues_path).await?;
    let frontmatter = IssueFrontmatter {
        display_number: new_display_number,
        status: source_issue.metadata.status.clone(),
        priority: source_issue.metadata.priority,
        created_at: source_issue.metadata.created_at.clone(),
        updated_at: now_iso(),
        draft: source_issue.metadata.draft,
        deleted_at: source_issue.metadata.deleted_at.clone(),
        is_org_issue: source_issue.metadata.is_org_issue,
        org_slug: source_issue.metadata.org_slug.clone(),
        org_display_number: source_issue.metadata.org_display_number,
        custom_fields: source_issue.metadata.custom_fields.clone(),
    };
    let target_issue_file = target_issues_path.join(format!("{}.md", &options.issue_id));
    let issue_content =
        generate_frontmatter(&frontmatter, &source_issue.title, &source_issue.description);
    fs::write(&target_issue_file, format_issue_file(&issue_content)).await?;
    let source_assets_path = if source_is_new_format {
        source_issues_path.join("assets").join(&options.issue_id)
    } else {
        source_folder_path.join("assets")
    };
    let target_assets_path = target_issues_path.join("assets").join(&options.issue_id);
    if source_assets_path.exists() {
        fs::create_dir_all(&target_assets_path).await?;
        copy_assets_folder(&source_assets_path, &target_assets_path)
            .await
            .map_err(|e| IssueCrudError::IoError(std::io::Error::other(e.to_string())))?;
    }
    if source_is_new_format {
        fs::remove_file(&source_file_path).await?;
        if source_assets_path.exists() {
            fs::remove_dir_all(&source_assets_path).await?;
        }
    } else {
        fs::remove_dir_all(&source_folder_path).await?;
    }
    update_manifest(&mut source_manifest);
    update_manifest(&mut target_manifest);
    write_manifest(&options.source_project_path, &source_manifest).await?;
    write_manifest(&options.target_project_path, &target_manifest).await?;
    let moved_issue = read_issue_from_frontmatter(&target_issue_file, &options.issue_id).await?;
    Ok(MoveIssueResult {
        issue: moved_issue,
        old_display_number,
        source_manifest,
        target_manifest,
    })
}
