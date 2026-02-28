//! Display number reconciliation for resolving conflicts.
use super::super::metadata::{IssueFrontmatter, IssueMetadata};
use super::scan::scan_issues;
use super::types::{IssueInfo, ReconcileError};
use crate::utils::{format_markdown, strip_centy_md_header};
use mdstore::{generate_frontmatter, parse_frontmatter};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
/// Reconcile display numbers to resolve conflicts.
///
/// Scans all issues, finds duplicate display numbers, and
/// reassigns them so each issue has a unique display number.
/// The oldest issue (by `created_at`) keeps its original number.
///
/// Returns the number of issues that were reassigned.
#[allow(
    unknown_lints,
    max_lines_per_function,
    clippy::too_many_lines,
    max_nesting_depth
)]
pub async fn reconcile_display_numbers(issues_path: &Path) -> Result<u32, ReconcileError> {
    if !issues_path.exists() {
        return Ok(0);
    }
    let issues = scan_issues(issues_path).await?;
    let mut by_display_number: HashMap<u32, Vec<&IssueInfo>> = HashMap::new();
    for issue in &issues {
        by_display_number
            .entry(issue.display_number)
            .or_default()
            .push(issue);
    }
    let max_display_number = issues.iter().map(|i| i.display_number).max().unwrap_or(0);
    let mut reassignments: Vec<(IssueInfo, u32)> = Vec::new();
    let mut next_available = max_display_number.saturating_add(1);
    for (display_number, mut group) in by_display_number {
        if group.len() <= 1 {
            continue;
        }
        if display_number == 0 {
            for issue in &group {
                reassignments.push(((*issue).clone(), next_available));
                next_available = next_available.saturating_add(1);
            }
            continue;
        }
        group.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        for issue in group.iter().skip(1) {
            reassignments.push(((*issue).clone(), next_available));
            next_available = next_available.saturating_add(1);
        }
    }
    let reassignment_count = reassignments.len() as u32;
    for (issue_info, new_display_number) in reassignments {
        if issue_info.is_new_format {
            let file_path = issues_path.join(format!("{}.md", issue_info.id));
            let content = fs::read_to_string(&file_path).await?;
            let (mut frontmatter, title, body): (IssueFrontmatter, String, String) =
                parse_frontmatter(strip_centy_md_header(&content))?;
            frontmatter.display_number = new_display_number;
            frontmatter.updated_at = crate::utils::now_iso();
            let new_content = generate_frontmatter(&frontmatter, &title, &body);
            fs::write(&file_path, format_markdown(&new_content)).await?;
        } else {
            let metadata_path = issues_path.join(&issue_info.id).join("metadata.json");
            let content = fs::read_to_string(&metadata_path).await?;
            let mut metadata: IssueMetadata = serde_json::from_str(&content)?;
            metadata.common.display_number = new_display_number;
            metadata.common.updated_at = crate::utils::now_iso();
            let new_content = serde_json::to_string_pretty(&metadata)?;
            fs::write(&metadata_path, new_content).await?;
        }
    }
    Ok(reassignment_count)
}
