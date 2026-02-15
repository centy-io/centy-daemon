use super::{CentyConfig, ConfigError, CustomFieldDefinition};
use crate::utils::get_centy_path;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeFeatures {
    pub display_number: bool,
    pub status: bool,
    pub priority: bool,
    pub soft_delete: bool,
    pub assets: bool,
    pub org_sync: bool,
    #[serde(rename = "move")]
    pub move_item: bool,
    pub duplicate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeConfig {
    pub name: String,
    pub plural: String,
    pub identifier: String,
    pub features: ItemTypeFeatures,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority_levels: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_fields: Vec<CustomFieldDefinition>,
}

/// Build the default issues config from the project's `CentyConfig`.
#[must_use]
pub fn default_issue_config(config: &CentyConfig) -> ItemTypeConfig {
    ItemTypeConfig {
        name: "Issue".to_string(),
        plural: "issues".to_string(),
        identifier: "uuid".to_string(),
        features: ItemTypeFeatures {
            display_number: true,
            status: true,
            priority: true,
            soft_delete: true,
            assets: true,
            org_sync: true,
            move_item: true,
            duplicate: true,
        },
        statuses: config.allowed_states.clone(),
        default_status: Some(config.default_state.clone()),
        priority_levels: Some(config.priority_levels),
        custom_fields: config.custom_fields.clone(),
    }
}

/// Build the default docs config with hardcoded defaults.
#[must_use]
pub fn default_doc_config() -> ItemTypeConfig {
    ItemTypeConfig {
        name: "Doc".to_string(),
        plural: "docs".to_string(),
        identifier: "slug".to_string(),
        features: ItemTypeFeatures {
            display_number: false,
            status: false,
            priority: false,
            soft_delete: true,
            assets: false,
            org_sync: true,
            move_item: true,
            duplicate: true,
        },
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
    }
}

/// Read an item-type config from `.centy/<folder>/config.yaml`.
#[allow(dead_code)] // Public API for future use
pub async fn read_item_type_config(
    project_path: &Path,
    folder: &str,
) -> Result<Option<ItemTypeConfig>, ConfigError> {
    let config_path = get_centy_path(project_path)
        .join(folder)
        .join("config.yaml");

    if !config_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&config_path).await?;
    let config: ItemTypeConfig = serde_yaml::from_str(&content)?;
    Ok(Some(config))
}

/// Write an item-type config to `.centy/<folder>/config.yaml`.
pub async fn write_item_type_config(
    project_path: &Path,
    folder: &str,
    config: &ItemTypeConfig,
) -> Result<(), ConfigError> {
    let dir_path = get_centy_path(project_path).join(folder);
    fs::create_dir_all(&dir_path).await?;

    let config_path = dir_path.join("config.yaml");
    let content = serde_yaml::to_string(config)?;
    fs::write(&config_path, content).await?;
    Ok(())
}

/// Discover all item types by scanning `.centy/*/config.yaml`.
///
/// Returns a list of `ItemTypeConfig` for each subdirectory that contains
/// a valid `config.yaml`.
#[allow(dead_code)] // Public API for future use
pub async fn discover_item_types(project_path: &Path) -> Result<Vec<ItemTypeConfig>, ConfigError> {
    let centy_path = get_centy_path(project_path);
    if !centy_path.exists() {
        return Ok(Vec::new());
    }

    let mut configs = Vec::new();
    let mut entries = fs::read_dir(&centy_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_dir() {
            continue;
        }

        let config_path = entry.path().join("config.yaml");
        if !config_path.exists() {
            continue;
        }

        let content = match fs::read_to_string(&config_path).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        if let Ok(config) = serde_yaml::from_str::<ItemTypeConfig>(&content) {
            configs.push(config);
        }
    }

    Ok(configs)
}

/// Create `config.yaml` for issues and docs if they don't already exist.
/// Returns the list of relative paths that were created.
pub async fn migrate_to_item_type_configs(
    project_path: &Path,
    config: &CentyConfig,
) -> Result<Vec<String>, ConfigError> {
    let mut created = Vec::new();

    let centy_path = get_centy_path(project_path);

    // Issues
    let issues_config_path = centy_path.join("issues").join("config.yaml");
    if !issues_config_path.exists() {
        let issue_config = default_issue_config(config);
        write_item_type_config(project_path, "issues", &issue_config).await?;
        created.push("issues/config.yaml".to_string());
    }

    // Docs
    let docs_config_path = centy_path.join("docs").join("config.yaml");
    if !docs_config_path.exists() {
        let doc_config = default_doc_config();
        write_item_type_config(project_path, "docs", &doc_config).await?;
        created.push("docs/config.yaml".to_string());
    }

    Ok(created)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::field_reassign_with_default
)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_issue_config_maps_fields() {
        let mut config = CentyConfig::default();
        config.allowed_states = vec![
            "open".to_string(),
            "in-progress".to_string(),
            "closed".to_string(),
        ];
        config.default_state = "open".to_string();
        config.priority_levels = 5;

        let issue = default_issue_config(&config);

        assert_eq!(issue.name, "Issue");
        assert_eq!(issue.plural, "issues");
        assert_eq!(issue.identifier, "uuid");
        assert_eq!(issue.statuses, config.allowed_states);
        assert_eq!(issue.default_status, Some("open".to_string()));
        assert_eq!(issue.priority_levels, Some(5));
        assert!(issue.features.display_number);
        assert!(issue.features.status);
        assert!(issue.features.priority);
        assert!(issue.features.soft_delete);
        assert!(issue.features.assets);
        assert!(issue.features.org_sync);
        assert!(issue.features.move_item);
        assert!(issue.features.duplicate);
    }

    #[test]
    fn test_default_doc_config() {
        let doc = default_doc_config();

        assert_eq!(doc.name, "Doc");
        assert_eq!(doc.plural, "docs");
        assert_eq!(doc.identifier, "slug");
        assert!(doc.statuses.is_empty());
        assert!(doc.default_status.is_none());
        assert!(doc.priority_levels.is_none());
        assert!(doc.custom_fields.is_empty());
        assert!(!doc.features.display_number);
        assert!(!doc.features.status);
        assert!(!doc.features.priority);
        assert!(doc.features.soft_delete);
        assert!(!doc.features.assets);
        assert!(doc.features.org_sync);
        assert!(doc.features.move_item);
        assert!(doc.features.duplicate);
    }

    #[test]
    fn test_issue_config_yaml_serialization() {
        let config = default_issue_config(&CentyConfig::default());
        let yaml = serde_yaml::to_string(&config).expect("Should serialize");

        assert!(yaml.contains("name: Issue"));
        assert!(yaml.contains("plural: issues"));
        assert!(yaml.contains("identifier: uuid"));
        assert!(yaml.contains("displayNumber: true"));
        assert!(yaml.contains("move: true"));
        assert!(yaml.contains("defaultStatus: open"));
    }

    #[test]
    fn test_doc_config_yaml_serialization() {
        let config = default_doc_config();
        let yaml = serde_yaml::to_string(&config).expect("Should serialize");

        assert!(yaml.contains("name: Doc"));
        assert!(yaml.contains("plural: docs"));
        assert!(yaml.contains("identifier: slug"));
        assert!(yaml.contains("displayNumber: false"));
        // Docs should NOT have statuses, defaultStatus, or priorityLevels
        assert!(!yaml.contains("statuses"));
        assert!(!yaml.contains("defaultStatus"));
        assert!(!yaml.contains("priorityLevels"));
    }

    #[test]
    fn test_item_type_config_yaml_roundtrip() {
        let config = default_issue_config(&CentyConfig::default());
        let yaml = serde_yaml::to_string(&config).expect("Should serialize");
        let deserialized: ItemTypeConfig = serde_yaml::from_str(&yaml).expect("Should deserialize");

        assert_eq!(deserialized.name, "Issue");
        assert_eq!(deserialized.statuses.len(), config.statuses.len());
        assert_eq!(deserialized.default_status, config.default_status);
        assert_eq!(deserialized.priority_levels, config.priority_levels);
    }

    #[tokio::test]
    async fn test_read_item_type_config_nonexistent() {
        let temp = tempdir().expect("Should create temp dir");
        let result = read_item_type_config(temp.path(), "issues")
            .await
            .expect("Should not error");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_write_and_read_item_type_config() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy").join("issues");
        fs::create_dir_all(&centy_dir)
            .await
            .expect("Should create dir");

        let config = default_issue_config(&CentyConfig::default());
        write_item_type_config(temp.path(), "issues", &config)
            .await
            .expect("Should write");

        let read = read_item_type_config(temp.path(), "issues")
            .await
            .expect("Should read")
            .expect("Should exist");

        assert_eq!(read.name, "Issue");
        assert_eq!(read.statuses, config.statuses);
    }

    #[tokio::test]
    async fn test_migrate_creates_both_configs() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");
        fs::create_dir_all(centy_dir.join("issues"))
            .await
            .expect("create issues/");
        fs::create_dir_all(centy_dir.join("docs"))
            .await
            .expect("create docs/");

        let config = CentyConfig::default();
        let created = migrate_to_item_type_configs(temp.path(), &config)
            .await
            .expect("Should migrate");

        assert_eq!(created.len(), 2);
        assert!(created.contains(&"issues/config.yaml".to_string()));
        assert!(created.contains(&"docs/config.yaml".to_string()));

        // Files should exist on disk
        assert!(centy_dir.join("issues").join("config.yaml").exists());
        assert!(centy_dir.join("docs").join("config.yaml").exists());
    }

    #[tokio::test]
    async fn test_migrate_skips_existing_configs() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");
        fs::create_dir_all(centy_dir.join("issues"))
            .await
            .expect("create issues/");
        fs::create_dir_all(centy_dir.join("docs"))
            .await
            .expect("create docs/");

        // Pre-create issues/config.yaml
        fs::write(
            centy_dir.join("issues").join("config.yaml"),
            "name: CustomIssue\n",
        )
        .await
        .expect("write");

        let config = CentyConfig::default();
        let created = migrate_to_item_type_configs(temp.path(), &config)
            .await
            .expect("Should migrate");

        // Only docs should be created
        assert_eq!(created.len(), 1);
        assert!(created.contains(&"docs/config.yaml".to_string()));

        // Existing file should be untouched
        let content = fs::read_to_string(centy_dir.join("issues").join("config.yaml"))
            .await
            .expect("read");
        assert_eq!(content, "name: CustomIssue\n");
    }

    #[test]
    fn test_issue_config_custom_fields_mapped() {
        let mut config = CentyConfig::default();
        config.custom_fields = vec![CustomFieldDefinition {
            name: "environment".to_string(),
            field_type: "enum".to_string(),
            required: true,
            default_value: Some("dev".to_string()),
            enum_values: vec!["dev".to_string(), "staging".to_string(), "prod".to_string()],
        }];

        let issue = default_issue_config(&config);
        assert_eq!(issue.custom_fields.len(), 1);
        assert_eq!(issue.custom_fields[0].name, "environment");
    }
}
