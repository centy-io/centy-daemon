//! Feature CRUD operations (WIP - not yet integrated)

use crate::item::entities::issue::{list_issues, Issue};
use crate::manifest::{read_manifest, update_manifest, write_manifest};
use crate::utils::{get_centy_path, now_iso};
use std::path::Path;
use thiserror::Error;
use tokio::fs;

use super::instruction::DEFAULT_INSTRUCTION_CONTENT;
use super::types::{CompactedIssueRef, FeatureStatus};

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

    // Count uncompacted issues
    let uncompacted_count = list_uncompacted_issues(project_path).await?.len() as u32;

    Ok(FeatureStatus {
        initialized,
        has_compact,
        has_instruction,
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
    update_manifest(&mut manifest);
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
    update_manifest(&mut manifest);
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
    update_manifest(&mut manifest);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::entities::issue::{Issue, IssueMetadataFlat};
    use std::collections::HashMap;

    fn create_test_issue(id: &str, display_number: u32, title: &str) -> Issue {
        #[allow(deprecated)]
        Issue {
            id: id.to_string(),
            issue_number: id.to_string(),
            title: title.to_string(),
            description: "Test description".to_string(),
            metadata: IssueMetadataFlat {
                display_number,
                status: "open".to_string(),
                priority: 1,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
                custom_fields: HashMap::new(),
                compacted: false,
                compacted_at: None,
                draft: false,
                deleted_at: None,
                is_org_issue: false,
                org_slug: None,
                org_display_number: None,
            },
        }
    }

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

    #[test]
    fn test_generate_migration_filename_special_chars() {
        let timestamp = "2025-12-06T00:00:00.000000+00:00";
        let filename = generate_migration_filename(timestamp);
        // Colons should be replaced with dashes
        assert!(!filename.contains(':'));
        assert!(std::path::Path::new(&filename)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md")));
    }

    #[test]
    fn test_build_compacted_refs_empty() {
        let issues: Vec<Issue> = vec![];
        let refs = build_compacted_refs(&issues);

        assert!(refs.is_empty());
    }

    #[test]
    fn test_build_compacted_refs_single_issue() {
        let issues = vec![create_test_issue("uuid-1", 1, "First Issue")];
        let refs = build_compacted_refs(&issues);

        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].id, "uuid-1");
        assert_eq!(refs[0].display_number, 1);
        assert_eq!(refs[0].title, "First Issue");
    }

    #[test]
    fn test_build_compacted_refs_multiple_issues() {
        let issues = vec![
            create_test_issue("uuid-1", 1, "First Issue"),
            create_test_issue("uuid-2", 2, "Second Issue"),
            create_test_issue("uuid-3", 3, "Third Issue"),
        ];
        let refs = build_compacted_refs(&issues);

        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].display_number, 1);
        assert_eq!(refs[1].display_number, 2);
        assert_eq!(refs[2].display_number, 3);
    }

    #[test]
    fn test_build_compacted_refs_preserves_order() {
        let issues = vec![
            create_test_issue("uuid-3", 3, "Third"),
            create_test_issue("uuid-1", 1, "First"),
            create_test_issue("uuid-2", 2, "Second"),
        ];
        let refs = build_compacted_refs(&issues);

        // Order should be preserved from input
        assert_eq!(refs[0].display_number, 3);
        assert_eq!(refs[1].display_number, 1);
        assert_eq!(refs[2].display_number, 2);
    }

    #[test]
    fn test_compacted_issue_ref_serialization() {
        let issue_ref = CompactedIssueRef {
            id: "test-uuid".to_string(),
            display_number: 42,
            title: "Test Title".to_string(),
        };

        let json = serde_json::to_string(&issue_ref).expect("Should serialize");
        assert!(json.contains("test-uuid"));
        assert!(json.contains("42"));
        assert!(json.contains("Test Title"));
        // Should use camelCase
        assert!(json.contains("displayNumber"));
    }

    #[test]
    fn test_compacted_issue_ref_deserialization() {
        let json = r#"{"id": "uuid", "displayNumber": 5, "title": "My Title"}"#;
        let issue_ref: CompactedIssueRef = serde_json::from_str(json).expect("Should deserialize");

        assert_eq!(issue_ref.id, "uuid");
        assert_eq!(issue_ref.display_number, 5);
        assert_eq!(issue_ref.title, "My Title");
    }

    #[test]
    fn test_feature_error_display() {
        let err = FeatureError::NotInitialized;
        let display = format!("{err}");
        assert!(display.contains("centy init"));

        let err = FeatureError::FeaturesNotInitialized;
        let display = format!("{err}");
        assert!(display.contains("Features not initialized"));
    }

    #[test]
    fn test_feature_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let feature_err = FeatureError::from(io_err);
        assert!(matches!(feature_err, FeatureError::IoError(_)));
    }
}
