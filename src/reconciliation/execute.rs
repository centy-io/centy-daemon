use super::managed_files::{get_managed_files, merge_json_content, MergeStrategy};
use super::plan::build_reconciliation_plan;
use crate::config::item_type_config::migrate_to_item_type_configs;
use crate::config::{read_config, write_config, CentyConfig};
use crate::manifest::{
    create_manifest, read_manifest, update_manifest, write_manifest, CentyManifest, ManagedFileType,
};
use crate::utils::get_centy_path;
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("Plan error: {0}")]
    PlanError(#[from] super::plan::PlanError),

    #[error("Config error: {0}")]
    ConfigError(#[from] mdstore::ConfigError),
}

/// User decisions for reconciliation
#[derive(Debug, Clone, Default)]
pub struct ReconciliationDecisions {
    /// Paths of files to restore
    pub restore: HashSet<String>,
    /// Paths of files to reset
    pub reset: HashSet<String>,
}

/// Result of reconciliation execution
#[derive(Debug, Clone, Default)]
pub struct ReconciliationResult {
    pub created: Vec<String>,
    pub restored: Vec<String>,
    pub reset: Vec<String>,
    pub skipped: Vec<String>,
    pub manifest: CentyManifest,
}

/// Execute the reconciliation plan
pub async fn execute_reconciliation(
    project_path: &Path,
    decisions: ReconciliationDecisions,
    force: bool,
) -> Result<ReconciliationResult, ExecuteError> {
    let centy_path = get_centy_path(project_path);
    let managed_templates = get_managed_files();

    // Create .centy directory if it doesn't exist
    if !centy_path.exists() {
        fs::create_dir_all(&centy_path).await?;
    }

    // Get or create manifest
    let mut manifest = read_manifest(project_path)
        .await?
        .unwrap_or_else(create_manifest);

    // Build the plan
    let plan = build_reconciliation_plan(project_path).await?;

    let mut result = ReconciliationResult::default();

    // Process files to create
    for file_info in &plan.to_create {
        create_file(&centy_path, &file_info.path, &managed_templates).await?;
        result.created.push(file_info.path.clone());
    }

    // Process files to restore
    for file_info in &plan.to_restore {
        if force || decisions.restore.contains(&file_info.path) {
            create_file(&centy_path, &file_info.path, &managed_templates).await?;
            result.restored.push(file_info.path.clone());
        } else {
            result.skipped.push(file_info.path.clone());
        }
    }

    // Process files to reset
    for file_info in &plan.to_reset {
        let template = managed_templates.get(&file_info.path);
        let has_merge = template
            .map(|t| t.merge_strategy.is_some())
            .unwrap_or(false);

        if has_merge {
            // Mergeable files are automatically merged without requiring user decision
            merge_file(&centy_path, &file_info.path, &managed_templates).await?;
            result.reset.push(file_info.path.clone());
        } else if decisions.reset.contains(&file_info.path) {
            create_file(&centy_path, &file_info.path, &managed_templates).await?;
            result.reset.push(file_info.path.clone());
        } else {
            result.skipped.push(file_info.path.clone());
        }
    }

    // Create config.json with defaults if it doesn't exist
    let config_path = centy_path.join("config.json");
    if !config_path.exists() {
        let default_config = CentyConfig::default();
        write_config(project_path, &default_config).await?;
        result.created.push("config.json".to_string());
    }

    // Create config.yaml for item type folders if they don't exist
    let config = read_config(project_path).await?.unwrap_or_default();
    let migrated = migrate_to_item_type_configs(project_path, &config).await?;
    for path in migrated {
        result.created.push(path);
    }

    // Update manifest timestamp and version
    update_manifest(&mut manifest);

    // Write manifest
    write_manifest(project_path, &manifest).await?;

    result.manifest = manifest;
    Ok(result)
}

/// Create a file or directory from template
async fn create_file(
    centy_path: &Path,
    relative_path: &str,
    templates: &std::collections::HashMap<String, super::managed_files::ManagedFileTemplate>,
) -> Result<(), ExecuteError> {
    let template = templates
        .get(relative_path)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Template not found"))?;

    let full_path = centy_path.join(relative_path.trim_end_matches('/'));

    match &template.file_type {
        ManagedFileType::Directory => {
            fs::create_dir_all(&full_path).await?;
        }
        ManagedFileType::File => {
            // Ensure parent directory exists
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            let content = template.content.as_deref().unwrap_or("");
            fs::write(&full_path, content).await?;
        }
    }

    Ok(())
}

/// Merge a file on disk with its template content using the template's merge strategy
async fn merge_file(
    centy_path: &Path,
    relative_path: &str,
    templates: &std::collections::HashMap<String, super::managed_files::ManagedFileTemplate>,
) -> Result<(), ExecuteError> {
    let template = templates
        .get(relative_path)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Template not found"))?;

    let full_path = centy_path.join(relative_path.trim_end_matches('/'));
    let template_content = template.content.as_deref().unwrap_or("");

    match &template.merge_strategy {
        Some(MergeStrategy::JsonArrayMerge) => {
            let existing_content = fs::read_to_string(&full_path).await?;
            let merged = merge_json_content(&existing_content, template_content)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            fs::write(&full_path, merged).await?;
        }
        None => {
            // No merge strategy — fall back to overwrite
            fs::write(&full_path, template_content).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_reconciliation_decisions_default() {
        let decisions = ReconciliationDecisions::default();

        assert!(decisions.restore.is_empty());
        assert!(decisions.reset.is_empty());
    }

    #[test]
    fn test_reconciliation_decisions_with_values() {
        let mut decisions = ReconciliationDecisions::default();
        decisions.restore.insert("README.md".to_string());
        decisions.reset.insert("config.json".to_string());

        assert!(decisions.restore.contains("README.md"));
        assert!(decisions.reset.contains("config.json"));
        assert_eq!(decisions.restore.len(), 1);
        assert_eq!(decisions.reset.len(), 1);
    }

    #[test]
    fn test_reconciliation_decisions_clone() {
        let mut decisions = ReconciliationDecisions::default();
        decisions.restore.insert("test.md".to_string());

        let cloned = decisions.clone();
        assert!(cloned.restore.contains("test.md"));
    }

    #[test]
    fn test_reconciliation_decisions_debug() {
        let decisions = ReconciliationDecisions::default();
        let debug_str = format!("{decisions:?}");
        assert!(debug_str.contains("ReconciliationDecisions"));
    }

    #[test]
    fn test_reconciliation_result_default() {
        let result = ReconciliationResult::default();

        assert!(result.created.is_empty());
        assert!(result.restored.is_empty());
        assert!(result.reset.is_empty());
        assert!(result.skipped.is_empty());
    }

    #[test]
    fn test_reconciliation_result_with_values() {
        let mut result = ReconciliationResult::default();
        result.created.push("README.md".to_string());
        result.restored.push("config.json".to_string());
        result.reset.push("issues/README.md".to_string());
        result.skipped.push("custom.md".to_string());

        assert_eq!(result.created.len(), 1);
        assert_eq!(result.restored.len(), 1);
        assert_eq!(result.reset.len(), 1);
        assert_eq!(result.skipped.len(), 1);
    }

    #[test]
    fn test_reconciliation_result_clone() {
        let mut result = ReconciliationResult::default();
        result.created.push("test.md".to_string());

        let cloned = result.clone();
        assert_eq!(cloned.created, vec!["test.md".to_string()]);
    }

    #[test]
    fn test_reconciliation_result_debug() {
        let result = ReconciliationResult::default();
        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("ReconciliationResult"));
    }

    #[test]
    fn test_execute_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let execute_err = ExecuteError::IoError(io_err);

        let display = format!("{execute_err}");
        assert!(display.contains("IO error"));
    }

    #[test]
    fn test_execute_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let execute_err = ExecuteError::from(io_err);

        assert!(matches!(execute_err, ExecuteError::IoError(_)));
    }

    #[tokio::test]
    async fn test_execute_reconciliation_creates_centy_folder() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let decisions = ReconciliationDecisions::default();

        execute_reconciliation(temp_dir.path(), decisions, false)
            .await
            .expect("Should execute reconciliation");

        // .centy folder should be created
        assert!(temp_dir.path().join(".centy").exists());
    }

    #[tokio::test]
    async fn test_execute_reconciliation_creates_managed_files() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let decisions = ReconciliationDecisions::default();

        let result = execute_reconciliation(temp_dir.path(), decisions, false)
            .await
            .expect("Should execute reconciliation");

        // Should have created files
        assert!(!result.created.is_empty());

        // Key directories should exist
        let centy_path = temp_dir.path().join(".centy");
        assert!(centy_path.join("issues").is_dir());
        assert!(centy_path.join("docs").is_dir());
        assert!(centy_path.join("assets").is_dir());
        assert!(centy_path.join("templates").is_dir());

        // Key files should exist
        assert!(centy_path.join("README.md").is_file());
        assert!(centy_path.join("cspell.json").is_file());
    }

    #[tokio::test]
    async fn test_execute_reconciliation_writes_manifest() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let decisions = ReconciliationDecisions::default();

        let result = execute_reconciliation(temp_dir.path(), decisions, false)
            .await
            .expect("Should execute reconciliation");

        // Manifest should exist
        let manifest_path = temp_dir.path().join(".centy").join(".centy-manifest.json");
        assert!(manifest_path.exists());

        // Result should have manifest populated
        assert_eq!(result.manifest.schema_version, 1);
        assert!(!result.manifest.centy_version.is_empty());
    }

    #[tokio::test]
    async fn test_execute_reconciliation_idempotent() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let decisions = ReconciliationDecisions::default();

        // First execution
        let result1 = execute_reconciliation(temp_dir.path(), decisions.clone(), false)
            .await
            .expect("Should execute first time");

        // Second execution
        let result2 =
            execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
                .await
                .expect("Should execute second time");

        // First execution should create files
        assert!(!result1.created.is_empty());

        // Second execution should not create anything (all up to date)
        assert!(result2.created.is_empty());
    }

    #[tokio::test]
    async fn test_execute_reconciliation_force_mode() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let decisions = ReconciliationDecisions::default();

        // Execute with force=true
        let result = execute_reconciliation(temp_dir.path(), decisions, true)
            .await
            .expect("Should execute with force");

        // Should have created files
        assert!(!result.created.is_empty());
    }

    #[tokio::test]
    async fn test_execute_reconciliation_skips_modified_without_decision() {
        use tempfile::tempdir;
        use tokio::fs as async_fs;

        let temp_dir = tempdir().expect("Should create temp dir");

        // First initialize
        execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
            .await
            .expect("Should initialize");

        // Modify a file
        let readme_path = temp_dir.path().join(".centy").join("README.md");
        async_fs::write(&readme_path, "Modified content")
            .await
            .expect("Should write");

        // Execute without reset decision
        let result =
            execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
                .await
                .expect("Should execute");

        // README should be in skipped (modified but no decision to reset)
        assert!(result.skipped.contains(&"README.md".to_string()));
    }

    #[tokio::test]
    async fn test_execute_reconciliation_creates_config_json() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let decisions = ReconciliationDecisions::default();

        let result = execute_reconciliation(temp_dir.path(), decisions, false)
            .await
            .expect("Should execute reconciliation");

        // config.json should be created
        let config_path = temp_dir.path().join(".centy").join("config.json");
        assert!(config_path.exists(), "config.json should be created");
        assert!(
            result.created.contains(&"config.json".to_string()),
            "Should report config.json as created"
        );

        // Should contain hooks key
        let content = fs::read_to_string(&config_path).await.expect("Should read");
        let value: serde_json::Value = serde_json::from_str(&content).expect("Should parse");
        assert!(
            value.as_object().unwrap().contains_key("hooks"),
            "config.json should contain hooks key"
        );
    }

    #[tokio::test]
    async fn test_execute_reconciliation_creates_issues_config_yaml() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let decisions = ReconciliationDecisions::default();

        let result = execute_reconciliation(temp_dir.path(), decisions, false)
            .await
            .expect("Should execute reconciliation");

        let config_path = temp_dir
            .path()
            .join(".centy")
            .join("issues")
            .join("config.yaml");
        assert!(config_path.exists(), "issues/config.yaml should be created");
        assert!(
            result.created.contains(&"issues/config.yaml".to_string()),
            "Should report issues/config.yaml as created"
        );
    }

    #[tokio::test]
    async fn test_execute_reconciliation_creates_docs_config_yaml() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let decisions = ReconciliationDecisions::default();

        let result = execute_reconciliation(temp_dir.path(), decisions, false)
            .await
            .expect("Should execute reconciliation");

        let config_path = temp_dir
            .path()
            .join(".centy")
            .join("docs")
            .join("config.yaml");
        assert!(config_path.exists(), "docs/config.yaml should be created");
        assert!(
            result.created.contains(&"docs/config.yaml".to_string()),
            "Should report docs/config.yaml as created"
        );
    }

    #[tokio::test]
    async fn test_execute_reconciliation_issues_config_yaml_content() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");

        execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
            .await
            .expect("Should execute reconciliation");

        let config_path = temp_dir
            .path()
            .join(".centy")
            .join("issues")
            .join("config.yaml");
        let content = fs::read_to_string(&config_path)
            .await
            .expect("Should read config.yaml");

        assert!(content.contains("name: Issue"), "Should have name: Issue");
        assert!(
            content.contains("identifier: uuid"),
            "Should use uuid identifier"
        );
        assert!(
            content.contains("displayNumber: true"),
            "Should have displayNumber enabled"
        );
        assert!(
            content.contains("status: true"),
            "Should have status enabled"
        );
        assert!(
            content.contains("priority: true"),
            "Should have priority enabled"
        );
        assert!(
            content.contains("defaultStatus: open"),
            "Should have defaultStatus: open"
        );
    }

    #[tokio::test]
    async fn test_execute_reconciliation_docs_config_yaml_content() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");

        execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
            .await
            .expect("Should execute reconciliation");

        let config_path = temp_dir
            .path()
            .join(".centy")
            .join("docs")
            .join("config.yaml");
        let content = fs::read_to_string(&config_path)
            .await
            .expect("Should read config.yaml");

        assert!(content.contains("name: Doc"), "Should have name: Doc");
        assert!(
            content.contains("identifier: slug"),
            "Should use slug identifier"
        );
        assert!(
            content.contains("displayNumber: false"),
            "Docs should not have displayNumber"
        );
        assert!(
            content.contains("status: false"),
            "Docs should not have status"
        );
        assert!(
            content.contains("priority: false"),
            "Docs should not have priority"
        );
        assert!(
            !content.contains("defaultStatus:"),
            "Docs should not have defaultStatus"
        );
        assert!(
            !content.contains("priorityLevels:"),
            "Docs should not have priorityLevels"
        );
    }

    #[tokio::test]
    async fn test_execute_reconciliation_does_not_overwrite_existing_config_yaml() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");

        // First init
        execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
            .await
            .expect("Should execute first init");

        // Overwrite issues/config.yaml with custom content
        let config_path = temp_dir
            .path()
            .join(".centy")
            .join("issues")
            .join("config.yaml");
        fs::write(&config_path, "name: CustomIssue\n")
            .await
            .expect("Should write custom config");

        // Re-init
        let result =
            execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
                .await
                .expect("Should execute second init");

        // Custom content should be preserved
        let content = fs::read_to_string(&config_path).await.expect("Should read");
        assert_eq!(
            content, "name: CustomIssue\n",
            "Existing config.yaml should not be overwritten"
        );

        // Should not be listed as created on re-init
        assert!(
            !result.created.contains(&"issues/config.yaml".to_string()),
            "Should not re-create existing issues/config.yaml"
        );
    }

    #[tokio::test]
    async fn test_execute_reconciliation_resets_with_decision() {
        use tempfile::tempdir;
        use tokio::fs as async_fs;

        let temp_dir = tempdir().expect("Should create temp dir");

        // First initialize
        execute_reconciliation(temp_dir.path(), ReconciliationDecisions::default(), false)
            .await
            .expect("Should initialize");

        // Modify a file
        let readme_path = temp_dir.path().join(".centy").join("README.md");
        async_fs::write(&readme_path, "Modified content")
            .await
            .expect("Should write");

        // Execute with reset decision
        let mut decisions = ReconciliationDecisions::default();
        decisions.reset.insert("README.md".to_string());

        let result = execute_reconciliation(temp_dir.path(), decisions, false)
            .await
            .expect("Should execute");

        // README should be reset
        assert!(result.reset.contains(&"README.md".to_string()));

        // Content should be restored to original
        let content = async_fs::read_to_string(&readme_path)
            .await
            .expect("Should read");
        assert!(content.contains("Centy Project")); // Original content
    }
}
