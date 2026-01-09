//! Display number reconciliation for resolving PR conflicts.
//!
//! When multiple users create PRs offline, they may assign the same display
//! number. This module detects and resolves such conflicts by:
//! 1. Keeping the oldest PR's display number (by `created_at`)
//! 2. Reassigning newer PRs to the next available number

use super::id::is_valid_pr_folder;
use super::metadata::PrMetadata;
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
}

/// Information about a PR needed for reconciliation
#[derive(Debug, Clone)]
struct PrInfo {
    folder_name: String,
    display_number: u32,
    created_at: String,
}

/// Reconcile display numbers to resolve conflicts.
///
/// This function scans all PRs, finds duplicate display numbers, and
/// reassigns them so each PR has a unique display number. The oldest
/// PR (by `created_at`) keeps its original number.
///
/// Returns the number of PRs that were reassigned.
#[allow(clippy::too_many_lines)]
pub async fn reconcile_pr_display_numbers(prs_path: &Path) -> Result<u32, ReconcileError> {
    if !prs_path.exists() {
        return Ok(0);
    }

    // Step 1: Read all PRs and their display numbers
    let mut prs: Vec<PrInfo> = Vec::new();
    let mut entries = fs::read_dir(prs_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_dir() {
            continue;
        }

        let folder_name = match entry.file_name().to_str() {
            Some(name) => name.to_string(),
            None => continue,
        };

        if !is_valid_pr_folder(&folder_name) {
            continue;
        }

        let metadata_path = entry.path().join("metadata.json");
        if !metadata_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&metadata_path).await?;
        let metadata: PrMetadata = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(_) => continue, // Skip malformed metadata
        };

        prs.push(PrInfo {
            folder_name,
            display_number: metadata.common.display_number,
            created_at: metadata.common.created_at,
        });
    }

    // Step 2: Find duplicates (group by display_number)
    let mut by_display_number: HashMap<u32, Vec<&PrInfo>> = HashMap::new();
    for pr in &prs {
        by_display_number
            .entry(pr.display_number)
            .or_default()
            .push(pr);
    }

    // Step 3: Find max display number for reassignment
    let max_display_number = prs.iter().map(|p| p.display_number).max().unwrap_or(0);

    // Step 4: Process duplicates
    let mut reassignments: Vec<(String, u32)> = Vec::new(); // (folder_name, new_display_number)
    let mut next_available = max_display_number + 1;

    for (display_number, mut group) in by_display_number {
        if group.len() <= 1 {
            continue; // No conflict
        }

        // Skip display_number 0 (PRs without display numbers)
        if display_number == 0 {
            // Assign each PR without display number a unique number
            for pr in &group {
                reassignments.push((pr.folder_name.clone(), next_available));
                next_available += 1;
            }
            continue;
        }

        // Sort by created_at (oldest first)
        group.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        // Keep the first (oldest), reassign the rest
        for pr in group.iter().skip(1) {
            reassignments.push((pr.folder_name.clone(), next_available));
            next_available += 1;
        }
    }

    // Step 5: Write reassignments
    let reassignment_count = reassignments.len() as u32;

    for (folder_name, new_display_number) in reassignments {
        let metadata_path = prs_path.join(&folder_name).join("metadata.json");
        let content = fs::read_to_string(&metadata_path).await?;
        let mut metadata: PrMetadata = serde_json::from_str(&content)?;

        metadata.common.display_number = new_display_number;
        metadata.common.updated_at = crate::utils::now_iso();

        let new_content = serde_json::to_string_pretty(&metadata)?;
        fs::write(&metadata_path, new_content).await?;
    }

    Ok(reassignment_count)
}

/// Get the next available display number.
///
/// Scans all existing PRs and returns max + 1.
pub async fn get_next_pr_display_number(prs_path: &Path) -> Result<u32, ReconcileError> {
    if !prs_path.exists() {
        return Ok(1);
    }

    let mut max_number: u32 = 0;
    let mut entries = fs::read_dir(prs_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_dir() {
            continue;
        }

        let folder_name = match entry.file_name().to_str() {
            Some(name) => name.to_string(),
            None => continue,
        };

        if !is_valid_pr_folder(&folder_name) {
            continue;
        }

        let metadata_path = entry.path().join("metadata.json");
        if !metadata_path.exists() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(&metadata_path).await {
            if let Ok(metadata) = serde_json::from_str::<PrMetadata>(&content) {
                max_number = max_number.max(metadata.common.display_number);
            }
        }
    }

    Ok(max_number + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_pr(
        prs_path: &Path,
        folder_name: &str,
        display_number: u32,
        created_at: &str,
    ) {
        let pr_path = prs_path.join(folder_name);
        fs::create_dir_all(&pr_path).await.unwrap();

        let metadata = serde_json::json!({
            "displayNumber": display_number,
            "status": "draft",
            "sourceBranch": "feature",
            "targetBranch": "main",
            "priority": 2,
            "createdAt": created_at,
            "updatedAt": created_at
        });

        fs::write(
            pr_path.join("metadata.json"),
            serde_json::to_string_pretty(&metadata).unwrap(),
        )
        .await
        .unwrap();

        fs::write(pr_path.join("pr.md"), "# Test PR\n")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_reconcile_no_conflicts() {
        let temp = TempDir::new().unwrap();
        let prs_path = temp.path().join("prs");
        fs::create_dir_all(&prs_path).await.unwrap();

        create_test_pr(
            &prs_path,
            "550e8400-e29b-41d4-a716-446655440001",
            1,
            "2024-01-01T10:00:00Z",
        )
        .await;
        create_test_pr(
            &prs_path,
            "550e8400-e29b-41d4-a716-446655440002",
            2,
            "2024-01-01T11:00:00Z",
        )
        .await;

        let reassigned = reconcile_pr_display_numbers(&prs_path).await.unwrap();
        assert_eq!(reassigned, 0);
    }

    #[tokio::test]
    async fn test_reconcile_with_conflict() {
        let temp = TempDir::new().unwrap();
        let prs_path = temp.path().join("prs");
        fs::create_dir_all(&prs_path).await.unwrap();

        // Both have display_number 4, but different created_at
        create_test_pr(
            &prs_path,
            "550e8400-e29b-41d4-a716-446655440001",
            4,
            "2024-01-01T10:00:00Z", // Older
        )
        .await;
        create_test_pr(
            &prs_path,
            "550e8400-e29b-41d4-a716-446655440002",
            4,
            "2024-01-01T10:05:00Z", // Newer
        )
        .await;
        create_test_pr(
            &prs_path,
            "550e8400-e29b-41d4-a716-446655440003",
            5,
            "2024-01-01T10:10:00Z",
        )
        .await;

        let reassigned = reconcile_pr_display_numbers(&prs_path).await.unwrap();
        assert_eq!(reassigned, 1);

        // Check the older one kept display_number 4
        let metadata1: PrMetadata = serde_json::from_str(
            &fs::read_to_string(
                prs_path
                    .join("550e8400-e29b-41d4-a716-446655440001")
                    .join("metadata.json"),
            )
            .await
            .unwrap(),
        )
        .unwrap();
        assert_eq!(metadata1.common.display_number, 4);

        // Check the newer one was reassigned to 6 (max was 5, so next is 6)
        let metadata2: PrMetadata = serde_json::from_str(
            &fs::read_to_string(
                prs_path
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
    async fn test_get_next_pr_display_number_empty() {
        let temp = TempDir::new().unwrap();
        let prs_path = temp.path().join("prs");

        let next = get_next_pr_display_number(&prs_path).await.unwrap();
        assert_eq!(next, 1);
    }

    #[tokio::test]
    async fn test_get_next_pr_display_number_with_existing() {
        let temp = TempDir::new().unwrap();
        let prs_path = temp.path().join("prs");
        fs::create_dir_all(&prs_path).await.unwrap();

        create_test_pr(
            &prs_path,
            "550e8400-e29b-41d4-a716-446655440001",
            5,
            "2024-01-01T10:00:00Z",
        )
        .await;

        let next = get_next_pr_display_number(&prs_path).await.unwrap();
        assert_eq!(next, 6);
    }
}
