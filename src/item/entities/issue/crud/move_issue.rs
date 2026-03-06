use super::super::assets::copy_assets_folder;
use super::super::metadata::IssueFrontmatter;
use super::super::reconcile::get_next_display_number;
use super::extra_types::{MoveIssueOptions, MoveIssueResult};
use super::move_io::{
    load_source_issue, remove_source_issue, source_issues_path, validate_issue_move,
};
use super::read::read_issue_from_frontmatter;
use super::types::{Issue, IssueCrudError};
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::{get_centy_path, now_iso, CENTY_HEADER_YAML};
use mdstore::generate_frontmatter;
use tokio::fs;

fn build_target_frontmatter(issue: &Issue, new_display_number: u32) -> IssueFrontmatter {
    IssueFrontmatter {
        display_number: new_display_number,
        status: issue.metadata.status.clone(),
        priority: issue.metadata.priority,
        created_at: issue.metadata.created_at.clone(),
        updated_at: now_iso(),
        draft: issue.metadata.draft,
        deleted_at: issue.metadata.deleted_at.clone(),
        is_org_issue: issue.metadata.is_org_issue,
        org_slug: issue.metadata.org_slug.clone(),
        org_display_number: issue.metadata.org_display_number,
        custom_fields: issue.metadata.custom_fields.clone(),
    }
}
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
    let src_issues = source_issues_path(&options.source_project_path);
    let (source_is_new_format, source_issue, source_file_path, source_folder_path) =
        load_source_issue(&src_issues, &options.issue_id).await?;
    let old_display_number = source_issue.metadata.display_number;
    validate_issue_move(&source_issue, &options.target_project_path).await?;
    let target_centy = get_centy_path(&options.target_project_path);
    let target_issues_path = target_centy.join("issues");
    fs::create_dir_all(&target_issues_path).await?;
    let new_display_number = get_next_display_number(&target_issues_path).await?;
    let frontmatter = build_target_frontmatter(&source_issue, new_display_number);
    let target_issue_file = target_issues_path.join(format!("{}.md", &options.issue_id));
    let issue_content = generate_frontmatter(
        &frontmatter,
        &source_issue.title,
        &source_issue.description,
        Some(CENTY_HEADER_YAML),
    );
    fs::write(&target_issue_file, &issue_content).await?;
    let source_assets_path = if source_is_new_format {
        src_issues.join("assets").join(&options.issue_id)
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
    remove_source_issue(
        source_is_new_format,
        &source_file_path,
        &source_assets_path,
        &source_folder_path,
    )
    .await?;
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
