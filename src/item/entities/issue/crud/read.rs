use super::parse::parse_issue_md;
use super::types::{Issue, IssueCrudError, IssueMetadataFlat};
use super::super::metadata::{IssueFrontmatter, IssueMetadata};
use super::super::planning::remove_planning_note;
use std::collections::HashMap;
use std::path::Path;
use mdstore::parse_frontmatter;
use tokio::fs;

pub async fn read_issue_from_frontmatter(
    issue_file_path: &Path,
    issue_id: &str,
) -> Result<Issue, IssueCrudError> {
    let content = fs::read_to_string(issue_file_path).await?;
    let (frontmatter, title, body): (IssueFrontmatter, String, String) =
        parse_frontmatter(&content)?;
    let description = remove_planning_note(&body);
    #[allow(deprecated)]
    Ok(Issue {
        id: issue_id.to_string(),
        issue_number: issue_id.to_string(),
        title,
        description,
        metadata: IssueMetadataFlat {
            display_number: frontmatter.display_number,
            status: frontmatter.status,
            priority: frontmatter.priority,
            created_at: frontmatter.created_at,
            updated_at: frontmatter.updated_at,
            custom_fields: frontmatter.custom_fields,
            draft: frontmatter.draft,
            deleted_at: frontmatter.deleted_at,
            is_org_issue: frontmatter.is_org_issue,
            org_slug: frontmatter.org_slug,
            org_display_number: frontmatter.org_display_number,
        },
    })
}

pub async fn read_issue_from_legacy_folder(
    issue_folder_path: &Path,
    issue_id: &str,
) -> Result<Issue, IssueCrudError> {
    let issue_md_path = issue_folder_path.join("issue.md");
    let metadata_path = issue_folder_path.join("metadata.json");
    if !issue_md_path.exists() || !metadata_path.exists() {
        return Err(IssueCrudError::InvalidIssueFormat(format!(
            "Issue {issue_id} is missing required files"
        )));
    }
    let issue_md = fs::read_to_string(&issue_md_path).await?;
    let (title, description) = parse_issue_md(&issue_md);
    let metadata_content = fs::read_to_string(&metadata_path).await?;
    let metadata: IssueMetadata = serde_json::from_str(&metadata_content)?;
    let custom_fields: HashMap<String, String> = metadata
        .common
        .custom_fields
        .into_iter()
        .map(|(k, v)| {
            let str_val = match v {
                serde_json::Value::String(s) => s,
                other => other.to_string(),
            };
            (k, str_val)
        })
        .collect();
    #[allow(deprecated)]
    Ok(Issue {
        id: issue_id.to_string(),
        issue_number: issue_id.to_string(),
        title,
        description,
        metadata: IssueMetadataFlat {
            display_number: metadata.common.display_number,
            status: metadata.common.status,
            priority: metadata.common.priority,
            created_at: metadata.common.created_at,
            updated_at: metadata.common.updated_at,
            custom_fields,
            draft: metadata.draft,
            deleted_at: metadata.deleted_at,
            is_org_issue: metadata.is_org_issue,
            org_slug: metadata.org_slug,
            org_display_number: metadata.org_display_number,
        },
    })
}
