//! Feature CRUD operations (WIP - not yet integrated)

use crate::item::entities::issue::{list_issues, Issue};
use crate::manifest::{read_manifest, update_manifest_timestamp, write_manifest};
use crate::utils::{get_centy_path, now_iso};
use std::path::Path;
use thiserror::Error;
use tokio::fs;

use super::instruction::DEFAULT_INSTRUCTION_CONTENT;
use super::types::{CompactedIssueRef, FeatureStatus, MigrationFrontmatter};

#[derive(Error, Debug)]
pub enum FeatureError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,

    #[error("Features not initialized")]
    FeaturesNotInitialized,

    #[error("Issue CRUD error: {0}")]
    IssueCrudError(#[from] crate::item::entities::issue::IssueCrudError),
}

/// Get the status of the features system
pub async fn get_feature_status(project_path: &Path) -> Result<FeatureStatus, FeatureError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(FeatureError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let features_path = centy_path.join("features");

    let initialized = features_path.exists();
    let has_compact = features_path.join("compact.md").exists();
    let has_instruction = features_path.join("instruction.md").exists();

    // Count migration files
    let migration_count = if features_path.join("migrations").exists() {
        count_migration_files(&features_path.join("migrations")).await?
    } else {
        0
    };

    // Count uncompacted issues
    let uncompacted_count = list_uncompacted_issues(project_path).await?.len() as u32;

    Ok(FeatureStatus {
        initialized,
        has_compact,
        has_instruction,
        migration_count,
        uncompacted_count,
    })
}

/// List all uncompacted issues
pub async fn list_uncompacted_issues(project_path: &Path) -> Result<Vec<Issue>, FeatureError> {
    let all_issues = list_issues(project_path, None, None, None, false).await?;

    let uncompacted: Vec<Issue> = all_issues
        .into_iter()
        .filter(|issue| !issue.metadata.compacted)
        .collect();

    Ok(uncompacted)
}

/// Get the current compact.md content
pub async fn get_compact(project_path: &Path) -> Result<Option<String>, FeatureError> {
    read_manifest(project_path)
        .await?
        .ok_or(FeatureError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let compact_path = centy_path.join("features").join("compact.md");

    if !compact_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&compact_path).await?;
    Ok(Some(content))
}

/// Update the compact.md content
pub async fn update_compact(project_path: &Path, content: &str) -> Result<(), FeatureError> {
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(FeatureError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let features_path = centy_path.join("features");

    if !features_path.exists() {
        return Err(FeatureError::FeaturesNotInitialized);
    }

    let compact_path = features_path.join("compact.md");
    fs::write(&compact_path, content).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    Ok(())
}

/// Get the instruction.md content
pub async fn get_instruction(project_path: &Path) -> Result<String, FeatureError> {
    read_manifest(project_path)
        .await?
        .ok_or(FeatureError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let instruction_path = centy_path.join("features").join("instruction.md");

    if !instruction_path.exists() {
        // Return default content if file doesn't exist
        return Ok(DEFAULT_INSTRUCTION_CONTENT.to_string());
    }

    let content = fs::read_to_string(&instruction_path).await?;
    Ok(content)
}

/// Save a migration file with timestamped filename
pub async fn save_migration(
    project_path: &Path,
    content: &str,
) -> Result<(String, String), FeatureError> {
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(FeatureError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let migrations_path = centy_path.join("features").join("migrations");

    if !migrations_path.exists() {
        fs::create_dir_all(&migrations_path).await?;
    }

    // Generate timestamp-based filename
    let timestamp = now_iso();
    let filename = generate_migration_filename(&timestamp);
    let migration_path = migrations_path.join(&filename);

    fs::write(&migration_path, content).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    let relative_path = format!(".centy/features/migrations/{filename}");
    Ok((filename, relative_path))
}

/// Mark multiple issues as compacted
pub async fn mark_issues_compacted(
    project_path: &Path,
    issue_ids: &[String],
) -> Result<u32, FeatureError> {
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(FeatureError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");
    let now = now_iso();
    let mut marked_count = 0;

    for issue_id in issue_ids {
        let metadata_path = issues_path.join(issue_id).join("metadata.json");

        if !metadata_path.exists() {
            continue;
        }

        // Read current metadata
        let content = fs::read_to_string(&metadata_path).await?;
        let mut metadata: crate::item::entities::issue::IssueMetadata =
            serde_json::from_str(&content)?;

        // Update compacted fields
        metadata.compacted = true;
        metadata.compacted_at = Some(now.clone());
        metadata.common.updated_at.clone_from(&now);

        // Write back
        fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?).await?;
        marked_count += 1;
    }

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    Ok(marked_count)
}

/// Generate migration filename from timestamp
fn generate_migration_filename(timestamp: &str) -> String {
    // Convert ISO timestamp to safe filename
    // 2025-12-06T19:30:00.123456+00:00 -> 2025-12-06T19-30-00.md
    let safe_ts = timestamp
        .chars()
        .take(19)
        .map(|c| if c == ':' { '-' } else { c })
        .collect::<String>();

    format!("{safe_ts}.md")
}

/// Count migration files in the migrations directory
async fn count_migration_files(migrations_path: &Path) -> Result<u32, std::io::Error> {
    if !migrations_path.exists() {
        return Ok(0);
    }

    let mut count = 0;
    let mut entries = fs::read_dir(migrations_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            count += 1;
        }
    }

    Ok(count)
}

/// Build compacted issue references from issues
#[must_use]
pub fn build_compacted_refs(issues: &[Issue]) -> Vec<CompactedIssueRef> {
    issues
        .iter()
        .map(|issue| CompactedIssueRef {
            id: issue.id.clone(),
            display_number: issue.metadata.display_number,
            title: issue.title.clone(),
        })
        .collect()
}

/// Generate migration frontmatter
#[must_use]
pub fn generate_migration_frontmatter(issues: &[Issue]) -> MigrationFrontmatter {
    MigrationFrontmatter {
        timestamp: now_iso(),
        compacted_issues: build_compacted_refs(issues),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_migration_filename() {
        let timestamp = "2025-12-06T19:30:00.123456+00:00";
        let filename = generate_migration_filename(timestamp);
        assert_eq!(filename, "2025-12-06T19-30-00.md");
    }

    #[test]
    fn test_generate_migration_filename_short() {
        let timestamp = "2025-12-06T19:30:00";
        let filename = generate_migration_filename(timestamp);
        assert_eq!(filename, "2025-12-06T19-30-00.md");
    }
}
