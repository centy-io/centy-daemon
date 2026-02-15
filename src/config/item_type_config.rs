use super::{CentyConfig, ConfigError, CustomFieldDefinition};
use crate::utils::get_centy_path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeFeatures {
    pub display_number: bool,
    pub status: bool,
    pub priority: bool,
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
            assets: true,
            org_sync: true,
            move_item: true,
            duplicate: true,
        },
        statuses: config.allowed_states.clone(),
        default_status: config.allowed_states.first().cloned(),
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
/// a valid `config.yaml`. Malformed configs are logged and skipped.
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

        let folder_name = entry.file_name().to_string_lossy().to_string();

        let content = match fs::read_to_string(&config_path).await {
            Ok(c) => c,
            Err(e) => {
                error!(folder = %folder_name, error = %e, "Failed to read config.yaml, skipping type");
                continue;
            }
        };

        match serde_yaml::from_str::<ItemTypeConfig>(&content) {
            Ok(config) => configs.push(config),
            Err(e) => {
                error!(folder = %folder_name, error = %e, "Malformed config.yaml, skipping type");
            }
        }
    }

    Ok(configs)
}

/// In-memory registry of item types built by scanning `.centy/*/config.yaml`.
///
/// Keyed by folder name (e.g. `"issues"`, `"docs"`).
#[derive(Debug, Clone)]
pub struct ItemTypeRegistry {
    types: HashMap<String, ItemTypeConfig>,
}

impl ItemTypeRegistry {
    /// Build the registry by scanning `.centy/*/config.yaml` files.
    ///
    /// * Scans `.centy/` direct subdirectories for `config.yaml` files
    /// * Parses each into an `ItemTypeConfig`
    /// * Validates no duplicate type names across folders
    /// * Skips directories without `config.yaml`
    /// * Logs errors for malformed configs and skips them (does not crash)
    /// * Logs discovered types at the end
    pub async fn build(project_path: &Path) -> Result<Self, ConfigError> {
        let centy_path = get_centy_path(project_path);
        let mut types = HashMap::new();

        if !centy_path.exists() {
            info!("No .centy directory found, item type registry is empty");
            return Ok(Self { types });
        }

        let mut entries = fs::read_dir(&centy_path).await?;
        // Track type names to detect duplicates: name -> folder that first registered it
        let mut seen_names: HashMap<String, String> = HashMap::new();

        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }

            let config_path = entry.path().join("config.yaml");
            if !config_path.exists() {
                continue;
            }

            let folder_name = entry.file_name().to_string_lossy().to_string();

            let content = match fs::read_to_string(&config_path).await {
                Ok(c) => c,
                Err(e) => {
                    error!(
                        folder = %folder_name,
                        error = %e,
                        "Failed to read config.yaml, skipping type"
                    );
                    continue;
                }
            };

            let config = match serde_yaml::from_str::<ItemTypeConfig>(&content) {
                Ok(c) => c,
                Err(e) => {
                    error!(
                        folder = %folder_name,
                        error = %e,
                        "Malformed config.yaml, skipping type"
                    );
                    continue;
                }
            };

            // Check for duplicate type names
            if let Some(existing_folder) = seen_names.get(&config.name) {
                warn!(
                    name = %config.name,
                    folder = %folder_name,
                    existing_folder = %existing_folder,
                    "Duplicate type name detected, skipping"
                );
                continue;
            }

            seen_names.insert(config.name.clone(), folder_name.clone());
            types.insert(folder_name, config);
        }

        let type_names: Vec<&str> = types.values().map(|c| c.name.as_str()).collect();
        info!(count = types.len(), types = ?type_names, "Item type registry built");

        Ok(Self { types })
    }

    /// Get a config by folder name (e.g. `"issues"`, `"docs"`).
    #[must_use]
    pub fn get(&self, folder: &str) -> Option<&ItemTypeConfig> {
        self.types.get(folder)
    }

    /// Get a config by type name (e.g. `"Issue"`, `"Doc"`).
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<(&String, &ItemTypeConfig)> {
        self.types.iter().find(|(_, c)| c.name == name)
    }

    /// Get all registered item types.
    #[must_use]
    pub fn all(&self) -> &HashMap<String, ItemTypeConfig> {
        &self.types
    }

    /// Get the number of registered types.
    #[must_use]
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Get folder names of all registered types.
    #[must_use]
    pub fn folders(&self) -> Vec<&String> {
        self.types.keys().collect()
    }

    /// Resolve an input string to a `(folder_name, config)` pair.
    ///
    /// Tries (in order):
    /// 1. Exact folder lookup (e.g. `"issues"`)
    /// 2. Case-insensitive name match (e.g. `"issue"` matches config with `name: "Issue"`)
    /// 3. Case-insensitive plural match (e.g. `"Issue"` matches config with `plural: "issues"`)
    #[must_use]
    pub fn resolve(&self, input: &str) -> Option<(&String, &ItemTypeConfig)> {
        // 1. Exact folder lookup
        if let Some((key, config)) = self.types.get_key_value(input) {
            return Some((key, config));
        }

        let input_lower = input.to_lowercase();

        // 2. Case-insensitive name match
        if let Some(pair) = self
            .types
            .iter()
            .find(|(_, c)| c.name.to_lowercase() == input_lower)
        {
            return Some(pair);
        }

        // 3. Case-insensitive plural match
        self.types
            .iter()
            .find(|(_, c)| c.plural.to_lowercase() == input_lower)
    }
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

    // ─── ItemTypeRegistry tests ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_registry_build_with_valid_configs() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");

        // Create issues config
        let issues_dir = centy_dir.join("issues");
        fs::create_dir_all(&issues_dir).await.unwrap();
        let issue_config = default_issue_config(&CentyConfig::default());
        let yaml = serde_yaml::to_string(&issue_config).unwrap();
        fs::write(issues_dir.join("config.yaml"), &yaml)
            .await
            .unwrap();

        // Create docs config
        let docs_dir = centy_dir.join("docs");
        fs::create_dir_all(&docs_dir).await.unwrap();
        let doc_config = default_doc_config();
        let yaml = serde_yaml::to_string(&doc_config).unwrap();
        fs::write(docs_dir.join("config.yaml"), &yaml)
            .await
            .unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());

        let issues = registry.get("issues").expect("Should have issues");
        assert_eq!(issues.name, "Issue");

        let docs = registry.get("docs").expect("Should have docs");
        assert_eq!(docs.name, "Doc");
    }

    #[tokio::test]
    async fn test_registry_build_skips_dirs_without_config() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");

        // Create one valid config
        let issues_dir = centy_dir.join("issues");
        fs::create_dir_all(&issues_dir).await.unwrap();
        let issue_config = default_issue_config(&CentyConfig::default());
        let yaml = serde_yaml::to_string(&issue_config).unwrap();
        fs::write(issues_dir.join("config.yaml"), &yaml)
            .await
            .unwrap();

        // Create directories without config.yaml (should be skipped)
        fs::create_dir_all(centy_dir.join("assets")).await.unwrap();
        fs::create_dir_all(centy_dir.join("templates"))
            .await
            .unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();
        assert_eq!(registry.len(), 1);
        assert!(registry.get("assets").is_none());
        assert!(registry.get("templates").is_none());
    }

    #[tokio::test]
    async fn test_registry_build_skips_malformed_yaml() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");

        // Create a valid config
        let issues_dir = centy_dir.join("issues");
        fs::create_dir_all(&issues_dir).await.unwrap();
        let issue_config = default_issue_config(&CentyConfig::default());
        let yaml = serde_yaml::to_string(&issue_config).unwrap();
        fs::write(issues_dir.join("config.yaml"), &yaml)
            .await
            .unwrap();

        // Create a malformed config.yaml
        let bad_dir = centy_dir.join("broken");
        fs::create_dir_all(&bad_dir).await.unwrap();
        fs::write(
            bad_dir.join("config.yaml"),
            "this is: [not: valid: yaml: {{",
        )
        .await
        .unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

        // Should have only the valid config, not crash
        assert_eq!(registry.len(), 1);
        assert!(registry.get("issues").is_some());
        assert!(registry.get("broken").is_none());
    }

    #[tokio::test]
    async fn test_registry_build_detects_duplicate_type_names() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");

        // Create two folders with the same type name
        let dir_a = centy_dir.join("aaa-issues");
        fs::create_dir_all(&dir_a).await.unwrap();
        let config_a = ItemTypeConfig {
            name: "Issue".to_string(),
            plural: "issues".to_string(),
            identifier: "uuid".to_string(),
            features: ItemTypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
        };
        let yaml = serde_yaml::to_string(&config_a).unwrap();
        fs::write(dir_a.join("config.yaml"), &yaml).await.unwrap();

        let dir_b = centy_dir.join("zzz-issues");
        fs::create_dir_all(&dir_b).await.unwrap();
        let config_b = ItemTypeConfig {
            name: "Issue".to_string(),
            plural: "alt-issues".to_string(),
            identifier: "uuid".to_string(),
            features: ItemTypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
        };
        let yaml = serde_yaml::to_string(&config_b).unwrap();
        fs::write(dir_b.join("config.yaml"), &yaml).await.unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

        // Only one of the two should be registered (first wins)
        assert_eq!(registry.len(), 1);

        // The one that was registered should have name "Issue"
        let (_, config) = registry.get_by_name("Issue").expect("Should have Issue");
        assert_eq!(config.name, "Issue");
    }

    #[tokio::test]
    async fn test_registry_build_empty_centy_dir() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");
        fs::create_dir_all(&centy_dir).await.unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[tokio::test]
    async fn test_registry_build_no_centy_dir() {
        let temp = tempdir().expect("Should create temp dir");

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();
        assert!(registry.is_empty());
    }

    #[tokio::test]
    async fn test_registry_get_by_name() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");

        let issues_dir = centy_dir.join("issues");
        fs::create_dir_all(&issues_dir).await.unwrap();
        let issue_config = default_issue_config(&CentyConfig::default());
        let yaml = serde_yaml::to_string(&issue_config).unwrap();
        fs::write(issues_dir.join("config.yaml"), &yaml)
            .await
            .unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

        let (folder, config) = registry.get_by_name("Issue").expect("Should find Issue");
        assert_eq!(folder, "issues");
        assert_eq!(config.name, "Issue");

        assert!(registry.get_by_name("NonExistent").is_none());
    }

    #[tokio::test]
    async fn test_registry_folders() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");

        let issues_dir = centy_dir.join("issues");
        fs::create_dir_all(&issues_dir).await.unwrap();
        let issue_config = default_issue_config(&CentyConfig::default());
        let yaml = serde_yaml::to_string(&issue_config).unwrap();
        fs::write(issues_dir.join("config.yaml"), &yaml)
            .await
            .unwrap();

        let docs_dir = centy_dir.join("docs");
        fs::create_dir_all(&docs_dir).await.unwrap();
        let doc_config = default_doc_config();
        let yaml = serde_yaml::to_string(&doc_config).unwrap();
        fs::write(docs_dir.join("config.yaml"), &yaml)
            .await
            .unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();
        let mut folders: Vec<&String> = registry.folders();
        folders.sort();

        assert_eq!(folders.len(), 2);
        assert_eq!(folders[0], "docs");
        assert_eq!(folders[1], "issues");
    }

    #[tokio::test]
    async fn test_registry_all() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");

        let issues_dir = centy_dir.join("issues");
        fs::create_dir_all(&issues_dir).await.unwrap();
        let issue_config = default_issue_config(&CentyConfig::default());
        let yaml = serde_yaml::to_string(&issue_config).unwrap();
        fs::write(issues_dir.join("config.yaml"), &yaml)
            .await
            .unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();
        let all = registry.all();
        assert_eq!(all.len(), 1);
        assert!(all.contains_key("issues"));
    }

    #[tokio::test]
    async fn test_registry_resolve_by_folder() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");

        let issues_dir = centy_dir.join("issues");
        fs::create_dir_all(&issues_dir).await.unwrap();
        let issue_config = default_issue_config(&CentyConfig::default());
        let yaml = serde_yaml::to_string(&issue_config).unwrap();
        fs::write(issues_dir.join("config.yaml"), &yaml)
            .await
            .unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

        // Exact folder name
        let (folder, config) = registry.resolve("issues").expect("Should resolve 'issues'");
        assert_eq!(folder, "issues");
        assert_eq!(config.name, "Issue");
    }

    #[tokio::test]
    async fn test_registry_resolve_by_name() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");

        let issues_dir = centy_dir.join("issues");
        fs::create_dir_all(&issues_dir).await.unwrap();
        let issue_config = default_issue_config(&CentyConfig::default());
        let yaml = serde_yaml::to_string(&issue_config).unwrap();
        fs::write(issues_dir.join("config.yaml"), &yaml)
            .await
            .unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

        // Case-insensitive name match
        let (folder, _) = registry.resolve("issue").expect("Should resolve 'issue'");
        assert_eq!(folder, "issues");

        let (folder, _) = registry.resolve("Issue").expect("Should resolve 'Issue'");
        assert_eq!(folder, "issues");

        let (folder, _) = registry.resolve("ISSUE").expect("Should resolve 'ISSUE'");
        assert_eq!(folder, "issues");
    }

    #[tokio::test]
    async fn test_registry_resolve_by_plural() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");

        // Create a custom type where folder != plural
        let epics_dir = centy_dir.join("my-epics");
        fs::create_dir_all(&epics_dir).await.unwrap();
        let epic_config = ItemTypeConfig {
            name: "Epic".to_string(),
            plural: "epics".to_string(),
            identifier: "uuid".to_string(),
            features: ItemTypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
        };
        let yaml = serde_yaml::to_string(&epic_config).unwrap();
        fs::write(epics_dir.join("config.yaml"), &yaml)
            .await
            .unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

        // Should resolve via plural
        let (folder, config) = registry.resolve("epics").expect("Should resolve 'epics'");
        assert_eq!(folder, "my-epics");
        assert_eq!(config.name, "Epic");

        // Should resolve via name
        let (folder, _) = registry.resolve("epic").expect("Should resolve 'epic'");
        assert_eq!(folder, "my-epics");
    }

    #[tokio::test]
    async fn test_registry_resolve_not_found() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");
        fs::create_dir_all(&centy_dir).await.unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();
        assert!(registry.resolve("nonexistent").is_none());
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
