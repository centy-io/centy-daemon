use super::super::metadata::IssueFrontmatter;
use super::super::planning::remove_planning_note;
use super::types::{Issue, IssueCrudError, IssueMetadataFlat};
use crate::utils::strip_centy_md_header;
use mdstore::parse_frontmatter;
use std::path::Path;
use tokio::fs;

pub async fn read_issue_from_frontmatter(
    issue_file_path: &Path,
    issue_id: &str,
) -> Result<Issue, IssueCrudError> {
    let content = fs::read_to_string(issue_file_path).await?;
    let (frontmatter, title, body): (IssueFrontmatter, String, String) =
        parse_frontmatter(strip_centy_md_header(&content))?;
    let description = remove_planning_note(&body);
    Ok(Issue {
        id: issue_id.to_string(),
        title,
        description,
        metadata: IssueMetadataFlat {
            display_number: frontmatter.display_number,
            status: frontmatter.status,
            priority: frontmatter.priority,
            created_at: frontmatter.created_at,
            custom_fields: frontmatter.custom_fields,
            draft: frontmatter.draft,
            deleted_at: frontmatter.deleted_at,
            projects: frontmatter.projects,
        },
    })
}
