//! Display number reconciliation for resolving conflicts.
//!
//! When multiple users create issues offline, they may assign the same display
//! number. This module detects and resolves such conflicts by:
//! 1. Keeping the oldest issue's display number (by `created_at`)
//! 2. Reassigning newer issues to the next available number

use super::id::{is_valid_issue_file, is_valid_issue_folder};
use super::metadata::{IssueFrontmatter, IssueMetadata};
use crate::common::{generate_frontmatter, parse_frontmatter};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum ReconcileError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("YAML frontmatter error: {0}")]
    FrontmatterError(#[from] crate::common::FrontmatterError),
}

/// Information about an issue needed for reconciliation
#[derive(Debug, Clone)]
struct IssueInfo {
    /// Issue ID (UUID)
    id: String,
    /// Whether this is a new format (.md file) or old format (folder)
    is_new_format: bool,
    display_number: u32,
    created_at: String,
}

/// Reconcile display numbers to resolve conflicts.
///
/// This function scans all issues, finds duplicate display numbers, and
/// reassigns them so each issue has a unique display number. The oldest
/// issue (by `created_at`) keeps its original number.
///
/// Returns the number of issues that were reassigned.
pub async fn reconcile_display_numbers(issues_path: &Path) -> Result<u32, ReconcileError> {
    if !issues_path.exists() {
        return Ok(0);
    }

    // Step 1: Read all issues and their display numbers (both formats)
    let mut issues: Vec<IssueInfo> = Vec::new();
    let mut entries = fs::read_dir(issues_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Check for new format: {uuid}.md file
        if file_type.is_file() && is_valid_issue_file(&name) {
            let content = match fs::read_to_string(entry.path()).await {
                Ok(c) => c,
                Err(_) => continue,
            };
            let frontmatter: IssueFrontmatter =
                match parse_frontmatter::<IssueFrontmatter>(&content) {
                    Ok((fm, _, _)) => fm,
                    Err(_) => continue, // Skip malformed files
                };
            let issue_id = name.trim_end_matches(".md").to_string();
            issues.push(IssueInfo {
                id: issue_id,
                is_new_format: true,
                display_number: frontmatter.display_number,
                created_at: frontmatter.created_at,
            });
        }
        // Check for old format: {uuid}/ folder
        else if file_type.is_dir() && is_valid_issue_folder(&name) {
            let metadata_path = entry.path().join("metadata.json");
            if !metadata_path.exists() {
                continue;
            }
            let content = match fs::read_to_string(&metadata_path).await {
                Ok(c) => c,
                Err(_) => continue,
            };
            let metadata: IssueMetadata = match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(_) => continue, // Skip malformed metadata
            };
            issues.push(IssueInfo {
                id: name,
                is_new_format: false,
                display_number: metadata.common.display_number,
                created_at: metadata.common.created_at,
            });
        }
    }

    // Step 2: Find duplicates (group by display_number)
    let mut by_display_number: HashMap<u32, Vec<&IssueInfo>> = HashMap::new();
    for issue in &issues {
        by_display_number
            .entry(issue.display_number)
            .or_default()
            .push(issue);
    }

    // Step 3: Find max display number for reassignment
    let max_display_number = issues.iter().map(|i| i.display_number).max().unwrap_or(0);

    // Step 4: Process duplicates
    let mut reassignments: Vec<(IssueInfo, u32)> = Vec::new(); // (issue_info, new_display_number)
    let mut next_available = max_display_number.saturating_add(1);

    for (display_number, mut group) in by_display_number {
        if group.len() <= 1 {
            continue; // No conflict
        }

        // Skip display_number 0 (legacy issues without display numbers)
        if display_number == 0 {
            // Assign each legacy issue a unique number
            for issue in &group {
                reassignments.push(((*issue).clone(), next_available));
                next_available = next_available.saturating_add(1);
            }
            continue;
        }

        // Sort by created_at (oldest first)
        group.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        // Keep the first (oldest), reassign the rest
        for issue in group.iter().skip(1) {
            reassignments.push(((*issue).clone(), next_available));
            next_available = next_available.saturating_add(1);
        }
    }

    // Step 5: Write reassignments
    let reassignment_count = reassignments.len() as u32;

    for (issue_info, new_display_number) in reassignments {
        if issue_info.is_new_format {
            // Update frontmatter file
            let file_path = issues_path.join(format!("{}.md", issue_info.id));
            let content = fs::read_to_string(&file_path).await?;
            let (mut frontmatter, title, body): (IssueFrontmatter, String, String) =
                parse_frontmatter(&content)?;
            frontmatter.display_number = new_display_number;
            frontmatter.updated_at = crate::utils::now_iso();
            let new_content = generate_frontmatter(&frontmatter, &title, &body);
            fs::write(&file_path, new_content).await?;
        } else {
            // Update legacy metadata.json
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

/// Get the next available display number.
///
/// Scans all existing issues (both formats) and returns max + 1.
pub async fn get_next_display_number(issues_path: &Path) -> Result<u32, ReconcileError> {
    if !issues_path.exists() {
        return Ok(1);
    }

    let mut max_number: u32 = 0;
    let mut entries = fs::read_dir(issues_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Check for new format: {uuid}.md file
        if file_type.is_file() && is_valid_issue_file(&name) {
            if let Ok(content) = fs::read_to_string(entry.path()).await {
                if let Ok((frontmatter, _, _)) = parse_frontmatter::<IssueFrontmatter>(&content) {
                    max_number = max_number.max(frontmatter.display_number);
                }
            }
        }
        // Check for old format: {uuid}/ folder
        else if file_type.is_dir() && is_valid_issue_folder(&name) {
            let metadata_path = entry.path().join("metadata.json");
            if !metadata_path.exists() {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&metadata_path).await {
                if let Ok(metadata) = serde_json::from_str::<IssueMetadata>(&content) {
                    max_number = max_number.max(metadata.common.display_number);
                }
            }
        }
    }

    Ok(max_number.saturating_add(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_issue(
        issues_path: &Path,
        folder_name: &str,
        display_number: u32,
        created_at: &str,
    ) {
        let issue_path = issues_path.join(folder_name);
        fs::create_dir_all(&issue_path).await.unwrap();

        let metadata = serde_json::json!({
            "displayNumber": display_number,
            "status": "open",
            "priority": 2,
            "createdAt": created_at,
            "updatedAt": created_at
        });

        fs::write(
            issue_path.join("metadata.json"),
            serde_json::to_string_pretty(&metadata).unwrap(),
        )
        .await
        .unwrap();

        fs::write(issue_path.join("issue.md"), "# Test Issue\n")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_reconcile_no_conflicts() {
        let temp = TempDir::new().unwrap();
        let issues_path = temp.path().join("issues");
        fs::create_dir_all(&issues_path).await.unwrap();

        create_test_issue(
            &issues_path,
            "550e8400-e29b-41d4-a716-446655440001",
            1,
            "2024-01-01T10:00:00Z",
        )
        .await;
        create_test_issue(
            &issues_path,
            "550e8400-e29b-41d4-a716-446655440002",
            2,
            "2024-01-01T11:00:00Z",
        )
        .await;

        let reassigned = reconcile_display_numbers(&issues_path).await.unwrap();
        assert_eq!(reassigned, 0);
    }

    #[tokio::test]
    async fn test_reconcile_with_conflict() {
        let temp = TempDir::new().unwrap();
        let issues_path = temp.path().join("issues");
        fs::create_dir_all(&issues_path).await.unwrap();

        // Both have display_number 4, but different created_at
        create_test_issue(
            &issues_path,
            "550e8400-e29b-41d4-a716-446655440001",
            4,
            "2024-01-01T10:00:00Z", // Older
        )
        .await;
        create_test_issue(
            &issues_path,
            "550e8400-e29b-41d4-a716-446655440002",
            4,
            "2024-01-01T10:05:00Z", // Newer
        )
        .await;
        create_test_issue(
            &issues_path,
            "550e8400-e29b-41d4-a716-446655440003",
            5,
            "2024-01-01T10:10:00Z",
        )
        .await;

        let reassigned = reconcile_display_numbers(&issues_path).await.unwrap();
        assert_eq!(reassigned, 1);

        // Check the older one kept display_number 4
        let metadata1: IssueMetadata = serde_json::from_str(
            &fs::read_to_string(
                issues_path
                    .join("550e8400-e29b-41d4-a716-446655440001")
                    .join("metadata.json"),
            )
            .await
            .unwrap(),
        )
        .unwrap();
        assert_eq!(metadata1.common.display_number, 4);

        // Check the newer one was reassigned to 6 (max was 5, so next is 6)
        let metadata2: IssueMetadata = serde_json::from_str(
            &fs::read_to_string(
                issues_path
                    .join("550e8400-e29b-41d4-a716-446655440002")
                    .join("metadata.json"),
            )
            .await
            .unwrap(),
        )
        .unwrap();
        assert_eq!(metadata2.common.display_number, 6);
    }

    #[tokio::test]
    async fn test_get_next_display_number_empty() {
        let temp = TempDir::new().unwrap();
        let issues_path = temp.path().join("issues");

        let next = get_next_display_number(&issues_path).await.unwrap();
        assert_eq!(next, 1);
    }

    #[tokio::test]
    async fn test_get_next_display_number_with_existing() {
        let temp = TempDir::new().unwrap();
        let issues_path = temp.path().join("issues");
        fs::create_dir_all(&issues_path).await.unwrap();

        create_test_issue(
            &issues_path,
            "550e8400-e29b-41d4-a716-446655440001",
            5,
            "2024-01-01T10:00:00Z",
        )
        .await;

        let next = get_next_display_number(&issues_path).await.unwrap();
        assert_eq!(next, 6);
    }
}
