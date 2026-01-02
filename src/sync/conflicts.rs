//! Conflict storage and resolution.
//!
//! This module handles storing and resolving merge conflicts
//! that occur during sync operations.

use super::SyncError;
use crate::utils::now_iso;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

/// Information about a merge conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// Unique conflict ID
    pub id: String,
    /// Type of item (issue, doc, pr)
    pub item_type: String,
    /// ID of the item with conflict
    pub item_id: String,
    /// Relative file path within .centy
    pub file_path: String,
    /// When the conflict was detected
    pub created_at: String,
    /// Base content (common ancestor), if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_content: Option<String>,
    /// Our version of the content
    pub ours_content: String,
    /// Their version of the content
    pub theirs_content: String,
    /// Description of the conflict
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ConflictInfo {
    /// Create a new conflict info
    pub fn new(
        item_type: &str,
        item_id: &str,
        file_path: &str,
        base_content: Option<String>,
        ours_content: String,
        theirs_content: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            item_type: item_type.to_string(),
            item_id: item_id.to_string(),
            file_path: file_path.to_string(),
            created_at: now_iso(),
            base_content,
            ours_content,
            theirs_content,
            description: None,
        }
    }

    /// Set a description for the conflict
    #[must_use]
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
}

/// How to resolve a conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConflictResolution {
    /// Take our version
    TakeOurs,
    /// Take their version
    TakeTheirs,
    /// Use custom merged content
    Merge { content: String },
}

/// Get the conflicts directory path
fn get_conflicts_dir(sync_path: &Path) -> PathBuf {
    sync_path.join(".conflicts")
}

/// Store a conflict for later resolution.
///
/// Returns the path where the conflict was stored.
pub async fn store_conflict(
    sync_path: &Path,
    item_type: &str,
    item_id: &str,
    conflict: ConflictInfo,
) -> Result<PathBuf, SyncError> {
    let conflicts_dir = get_conflicts_dir(sync_path);
    fs::create_dir_all(&conflicts_dir).await?;

    let conflict_file = conflicts_dir.join(format!("{}.json", conflict.id));
    let content = serde_json::to_string_pretty(&conflict)?;
    fs::write(&conflict_file, content).await?;

    // Also create a summary file for the item
    let item_conflicts_file = conflicts_dir.join(format!("{item_type}_{item_id}.txt"));
    let summary = format!(
        "Conflict: {}\nFile: {}\nCreated: {}\n",
        conflict.id, conflict.file_path, conflict.created_at
    );

    // Append to summary file
    let mut existing = fs::read_to_string(&item_conflicts_file)
        .await
        .unwrap_or_default();
    existing.push_str(&summary);
    fs::write(&item_conflicts_file, existing).await?;

    Ok(conflict_file)
}

/// List all unresolved conflicts
pub async fn list_conflicts(sync_path: &Path) -> Result<Vec<ConflictInfo>, SyncError> {
    let conflicts_dir = get_conflicts_dir(sync_path);

    if !conflicts_dir.exists() {
        return Ok(Vec::new());
    }

    let mut conflicts = Vec::new();
    let mut entries = fs::read_dir(&conflicts_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            if let Ok(content) = fs::read_to_string(&path).await {
                if let Ok(conflict) = serde_json::from_str::<ConflictInfo>(&content) {
                    conflicts.push(conflict);
                }
            }
        }
    }

    // Sort by creation time (newest first)
    conflicts.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(conflicts)
}

/// Get a specific conflict by ID
pub async fn get_conflict(
    sync_path: &Path,
    conflict_id: &str,
) -> Result<Option<ConflictInfo>, SyncError> {
    let conflicts_dir = get_conflicts_dir(sync_path);
    let conflict_file = conflicts_dir.join(format!("{conflict_id}.json"));

    if !conflict_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&conflict_file).await?;
    let conflict = serde_json::from_str(&content)?;

    Ok(Some(conflict))
}

/// Resolve a conflict with the given resolution.
///
/// This applies the resolution to the actual file and removes the conflict record.
pub async fn resolve_conflict(
    sync_path: &Path,
    conflict_id: &str,
    resolution: ConflictResolution,
) -> Result<(), SyncError> {
    let conflicts_dir = get_conflicts_dir(sync_path);
    let conflict_file = conflicts_dir.join(format!("{conflict_id}.json"));

    if !conflict_file.exists() {
        return Err(SyncError::ConflictNotFound(conflict_id.to_string()));
    }

    // Read the conflict
    let content = fs::read_to_string(&conflict_file).await?;
    let conflict: ConflictInfo = serde_json::from_str(&content)?;

    // Determine the resolved content
    let resolved_content = match resolution {
        ConflictResolution::TakeOurs => conflict.ours_content.clone(),
        ConflictResolution::TakeTheirs => conflict.theirs_content.clone(),
        ConflictResolution::Merge { content } => content,
    };

    // Apply the resolution to the file
    let file_path = sync_path.join(&conflict.file_path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(&file_path, resolved_content).await?;

    // Remove the conflict record
    fs::remove_file(&conflict_file).await?;

    // Clean up summary file
    let summary_file = conflicts_dir.join(format!(
        "{}_{}.txt",
        conflict.item_type, conflict.item_id
    ));
    if summary_file.exists() {
        // Remove the line for this conflict from the summary
        if let Ok(summary_content) = fs::read_to_string(&summary_file).await {
            let new_content: String = summary_content
                .lines()
                .filter(|line| !line.contains(&conflict.id))
                .collect::<Vec<_>>()
                .join("\n");
            if new_content.trim().is_empty() {
                fs::remove_file(&summary_file).await?;
            } else {
                fs::write(&summary_file, new_content).await?;
            }
        }
    }

    Ok(())
}

/// Get conflicts for a specific item
pub async fn get_conflicts_for_item(
    sync_path: &Path,
    item_type: &str,
    item_id: &str,
) -> Result<Vec<ConflictInfo>, SyncError> {
    let all_conflicts = list_conflicts(sync_path).await?;

    Ok(all_conflicts
        .into_iter()
        .filter(|c| c.item_type == item_type && c.item_id == item_id)
        .collect())
}

/// Check if an item has any unresolved conflicts
pub async fn has_conflicts(
    sync_path: &Path,
    item_type: &str,
    item_id: &str,
) -> Result<bool, SyncError> {
    let conflicts = get_conflicts_for_item(sync_path, item_type, item_id).await?;
    Ok(!conflicts.is_empty())
}

/// Clear all conflicts (use with caution)
pub async fn clear_all_conflicts(sync_path: &Path) -> Result<usize, SyncError> {
    let conflicts_dir = get_conflicts_dir(sync_path);

    if !conflicts_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    let mut entries = fs::read_dir(&conflicts_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        if fs::remove_file(entry.path()).await.is_ok() {
            count += 1;
        }
    }

    // Try to remove the directory if empty
    let _ = fs::remove_dir(&conflicts_dir).await;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_info_new() {
        let conflict = ConflictInfo::new(
            "issue",
            "abc123",
            ".centy/issues/abc123/issue.md",
            Some("base content".to_string()),
            "our content".to_string(),
            "their content".to_string(),
        );

        assert_eq!(conflict.item_type, "issue");
        assert_eq!(conflict.item_id, "abc123");
        assert!(!conflict.id.is_empty());
        assert!(!conflict.created_at.is_empty());
    }

    #[test]
    fn test_conflict_info_with_description() {
        let conflict = ConflictInfo::new(
            "issue",
            "abc123",
            ".centy/issues/abc123/issue.md",
            None,
            "our content".to_string(),
            "their content".to_string(),
        )
        .with_description("Both sides modified the title");

        assert_eq!(
            conflict.description,
            Some("Both sides modified the title".to_string())
        );
    }

    #[test]
    fn test_conflict_resolution_serialization() {
        let resolution = ConflictResolution::TakeOurs;
        let json = serde_json::to_string(&resolution).unwrap();
        assert!(json.contains("take_ours"));

        let resolution = ConflictResolution::Merge {
            content: "merged".to_string(),
        };
        let json = serde_json::to_string(&resolution).unwrap();
        assert!(json.contains("merge"));
        assert!(json.contains("merged"));
    }
}
