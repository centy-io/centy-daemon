use super::CentyConfig;
use crate::utils::get_centy_path;
use mdstore::{CustomFieldDef, IdStrategy, TypeConfig, TypeFeatures};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{error, info, warn};

// ── Schema types ──────────────────────────────────────────────────────────────

/// Features that can be toggled per item type.
///
/// Stored as a nested object inside `config.yaml` under the `features` key.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeFeatures {
    /// Enable display numbers (1, 2, 3…) for items.
    #[serde(default)]
    pub display_number: bool,
    /// Enable status tracking (e.g. `open`, `in-progress`, `closed`).
    #[serde(default)]
    pub status: bool,
    /// Enable priority levels.
    #[serde(default)]
    pub priority: bool,
    /// Enable soft-deletion (items can be deleted and restored).
    #[serde(default)]
    pub soft_delete: bool,
    /// Enable file attachments.
    #[serde(default)]
    pub assets: bool,
    /// Enable organization sync.
    #[serde(default)]
    pub org_sync: bool,
    /// Enable moving items between projects.
    #[serde(rename = "move", default)]
    pub move_item: bool,
    /// Enable duplication of items.
    #[serde(default)]
    pub duplicate: bool,
}

/// Full configuration for an item type, parsed from `.centy/<folder>/config.yaml`.
///
/// This is the canonical schema for item-type configuration in centy-daemon.
/// Both built-in types (`issues`, `docs`) and custom types share this format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeConfig {
    /// Human-readable singular name (e.g. `"Issue"`, `"Doc"`, `"Epic"`).
    pub name: String,
    /// Optional icon identifier (e.g. `"clipboard"`, `"document"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// ID generation strategy: `uuid` (distributed) or `slug` (title-derived).
    pub identifier: IdStrategy,
    /// Toggle-able features for this item type.
    pub features: ItemTypeFeatures,
    /// Allowed status values (e.g. `["open", "in-progress", "closed"]`).
    /// Omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<String>,
    /// Default status for newly created items. Must appear in `statuses`.
    /// Omitted when not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_status: Option<String>,
    /// Number of priority levels (must be > 0 when present).
    /// Omitted when not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority_levels: Option<u32>,
    /// Custom field definitions for this item type.
    /// Omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_fields: Vec<CustomFieldDef>,
    /// Path to the Handlebars template file, relative to `.centy/<folder>/`.
    /// Omitted when not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Validate an `ItemTypeConfig` for correctness.
///
/// Checks:
/// - `name` must not be empty or whitespace-only.
/// - `priorityLevels` must be > 0 when present.
/// - Every value in `statuses` must be non-empty (after trimming).
/// - `defaultStatus` must appear in `statuses` when both are set.
pub fn validate_item_type_config(config: &ItemTypeConfig) -> Result<(), String> {
    if config.name.trim().is_empty() {
        return Err("name must not be empty".to_string());
    }
    if let Some(levels) = config.priority_levels {
        if levels == 0 {
            return Err("priorityLevels must be greater than 0".to_string());
        }
    }
    for status in &config.statuses {
        if status.trim().is_empty() {
            return Err("status names must not be empty".to_string());
        }
    }
    if let Some(default) = &config.default_status {
        if !config.statuses.contains(default) {
            return Err(format!(
                "defaultStatus \"{default}\" must be in statuses list"
            ));
        }
    }
    Ok(())
}

// ── mdstore conversion ────────────────────────────────────────────────────────

/// Convert an `ItemTypeConfig` to mdstore's `TypeConfig` for storage operations.
///
/// The `icon`, `soft_delete`, and `template` fields are centy-daemon-only
/// metadata and are intentionally dropped in this conversion.
impl From<&ItemTypeConfig> for TypeConfig {
    fn from(config: &ItemTypeConfig) -> TypeConfig {
        TypeConfig {
            name: config.name.clone(),
            identifier: config.identifier,
            features: TypeFeatures {
                display_number: config.features.display_number,
                status: config.features.status,
                priority: config.features.priority,
                assets: config.features.assets,
                org_sync: config.features.org_sync,
                move_item: config.features.move_item,
                duplicate: config.features.duplicate,
            },
            statuses: config.statuses.clone(),
            default_status: config.default_status.clone(),
            priority_levels: config.priority_levels,
            custom_fields: config.custom_fields.clone(),
        }
    }
}

// ── Default configs ───────────────────────────────────────────────────────────

/// Build the default issues config from the project's `CentyConfig`.
#[must_use]
pub fn default_issue_config(config: &CentyConfig) -> ItemTypeConfig {
    ItemTypeConfig {
        name: "Issue".to_string(),
        icon: Some("clipboard".to_string()),
        identifier: IdStrategy::Uuid,
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
        default_status: config.allowed_states.first().cloned(),
        priority_levels: Some(config.priority_levels),
        custom_fields: config.custom_fields.clone(),
        template: Some("template.md".to_string()),
    }
}

/// Build the default docs config with hardcoded defaults.
#[must_use]
pub fn default_doc_config() -> ItemTypeConfig {
    ItemTypeConfig {
        name: "Doc".to_string(),
        icon: Some("document".to_string()),
        identifier: IdStrategy::Slug,
        features: ItemTypeFeatures {
            display_number: false,
            status: false,
            priority: false,
            soft_delete: false,
            assets: false,
            org_sync: true,
            move_item: true,
            duplicate: true,
        },
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: Vec::new(),
        template: None,
    }
}

/// Build the default archived config with hardcoded defaults.
///
/// The archived folder is a catch-all for items moved out of active view.
/// Items retain their content and metadata; `original_item_type` tracks the
/// source folder so they can be unarchived back to the correct location.
#[must_use]
pub fn default_archived_config() -> ItemTypeConfig {
    ItemTypeConfig {
        name: "Archived".to_string(),
        icon: None,
        identifier: IdStrategy::Uuid,
        features: ItemTypeFeatures {
            display_number: false,
            status: false,
            priority: false,
            soft_delete: false,
            assets: true,
            org_sync: true,
            move_item: true,
            duplicate: false,
        },
        statuses: Vec::new(),
        default_status: None,
        priority_levels: None,
        custom_fields: vec![CustomFieldDef {
            name: "original_item_type".to_string(),
            field_type: "string".to_string(),
            required: false,
            default_value: None,
            enum_values: Vec::new(),
        }],
        template: None,
    }
}

// ── File I/O ──────────────────────────────────────────────────────────────────

/// Scan `.centy/*/config.yaml` and return a map of folder → `ItemTypeConfig`.
///
/// Malformed YAML files are logged and skipped; the function does not fail.
async fn discover_item_types_map(
    centy_path: &Path,
) -> Result<HashMap<String, ItemTypeConfig>, mdstore::ConfigError> {
    if !centy_path.exists() {
        return Ok(HashMap::new());
    }

    let mut configs = HashMap::new();
    let mut entries = fs::read_dir(centy_path).await?;

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
            Ok(config) => {
                configs.insert(folder_name, config);
            }
            Err(e) => {
                error!(folder = %folder_name, error = %e, "Malformed config.yaml, skipping type");
            }
        }
    }

    Ok(configs)
}

/// Read an item-type config from `.centy/<folder>/config.yaml`.
pub async fn read_item_type_config(
    project_path: &Path,
    folder: &str,
) -> Result<Option<ItemTypeConfig>, mdstore::ConfigError> {
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
///
/// Creates the directory if it does not already exist.
pub async fn write_item_type_config(
    project_path: &Path,
    folder: &str,
    config: &ItemTypeConfig,
) -> Result<(), mdstore::ConfigError> {
    let type_dir = get_centy_path(project_path).join(folder);
    fs::create_dir_all(&type_dir).await?;
    let config_path = type_dir.join("config.yaml");
    let content = serde_yaml::to_string(config)?;
    fs::write(&config_path, content).await?;
    Ok(())
}

/// Discover all item types by scanning `.centy/*/config.yaml`.
///
/// Returns a list of `ItemTypeConfig` for each subdirectory that contains
/// a valid `config.yaml`. Malformed configs are logged and skipped.
pub async fn discover_item_types(
    project_path: &Path,
) -> Result<Vec<ItemTypeConfig>, mdstore::ConfigError> {
    let centy_path = get_centy_path(project_path);
    Ok(discover_item_types_map(&centy_path)
        .await?
        .into_values()
        .collect())
}

// ── Registry ──────────────────────────────────────────────────────────────────

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
    pub async fn build(project_path: &Path) -> Result<Self, mdstore::ConfigError> {
        let centy_path = get_centy_path(project_path);
        let types = discover_item_types_map(&centy_path).await?;

        // Duplicate name detection (centy-specific logic)
        let mut seen_names: HashMap<String, String> = HashMap::new();
        let mut deduped = HashMap::new();
        for (folder, config) in types {
            if let Some(existing) = seen_names.get(&config.name) {
                warn!(
                    name = %config.name,
                    folder = %folder,
                    existing_folder = %existing,
                    "Duplicate type name detected, skipping"
                );
                continue;
            }
            seen_names.insert(config.name.clone(), folder.clone());
            deduped.insert(folder, config);
        }

        let type_names: Vec<&str> = deduped.values().map(|c| c.name.as_str()).collect();
        info!(count = deduped.len(), types = ?type_names, "Item type registry built");

        Ok(Self { types: deduped })
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
    /// 3. Case-insensitive folder match (e.g. `"Issues"` matches folder `"issues"`)
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

        // 3. Case-insensitive folder match
        self.types
            .iter()
            .find(|(k, _)| k.to_lowercase() == input_lower)
    }
}

// ── Migration ─────────────────────────────────────────────────────────────────

/// Create `config.yaml` for issues, docs, and archived if they don't already exist.
/// Returns the list of relative paths that were created.
pub async fn migrate_to_item_type_configs(
    project_path: &Path,
    config: &CentyConfig,
) -> Result<Vec<String>, mdstore::ConfigError> {
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

    // Archived
    let archived_config_path = centy_path.join("archived").join("config.yaml");
    if !archived_config_path.exists() {
        let archived_config = default_archived_config();
        write_item_type_config(project_path, "archived", &archived_config).await?;
        created.push("archived/config.yaml".to_string());
    }

    Ok(created)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::field_reassign_with_default
)]
mod tests {
    use super::*;
    use mdstore::CustomFieldDef;
    use tempfile::tempdir;
    use tokio::fs;

    // ─── Default config tests ─────────────────────────────────────────────────

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
        assert_eq!(issue.icon, Some("clipboard".to_string()));
        assert_eq!(issue.identifier, IdStrategy::Uuid);
        assert_eq!(issue.statuses, config.allowed_states);
        assert_eq!(issue.default_status, Some("open".to_string()));
        assert_eq!(issue.priority_levels, Some(5));
        assert_eq!(issue.template, Some("template.md".to_string()));
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
        assert_eq!(doc.icon, Some("document".to_string()));
        assert_eq!(doc.identifier, IdStrategy::Slug);
        assert!(doc.statuses.is_empty());
        assert!(doc.default_status.is_none());
        assert!(doc.priority_levels.is_none());
        assert!(doc.custom_fields.is_empty());
        assert!(doc.template.is_none());
        assert!(!doc.features.display_number);
        assert!(!doc.features.status);
        assert!(!doc.features.priority);
        assert!(!doc.features.soft_delete);
        assert!(!doc.features.assets);
        assert!(doc.features.org_sync);
        assert!(doc.features.move_item);
        assert!(doc.features.duplicate);
    }

    #[test]
    fn test_default_archived_config() {
        let archived = default_archived_config();

        assert_eq!(archived.name, "Archived");
        assert!(archived.icon.is_none());
        assert_eq!(archived.identifier, IdStrategy::Uuid);
        assert!(archived.statuses.is_empty());
        assert!(archived.default_status.is_none());
        assert!(archived.priority_levels.is_none());
        assert!(archived.template.is_none());
        assert_eq!(archived.custom_fields.len(), 1);
        assert_eq!(archived.custom_fields[0].name, "original_item_type");
        assert_eq!(archived.custom_fields[0].field_type, "string");
        assert!(!archived.features.display_number);
        assert!(!archived.features.status);
        assert!(!archived.features.priority);
        assert!(!archived.features.soft_delete);
        assert!(archived.features.assets);
        assert!(archived.features.org_sync);
        assert!(archived.features.move_item);
        assert!(!archived.features.duplicate);
    }

    // ─── Serialization tests ──────────────────────────────────────────────────

    #[test]
    fn test_archived_config_yaml_serialization() {
        let config = default_archived_config();
        let yaml = serde_yaml::to_string(&config).expect("Should serialize");

        assert!(yaml.contains("name: Archived"));
        assert!(yaml.contains("identifier: uuid"));
        assert!(yaml.contains("displayNumber: false"));
        assert!(yaml.contains("status: false"));
        assert!(yaml.contains("priority: false"));
        assert!(yaml.contains("softDelete: false"));
        assert!(yaml.contains("assets: true"));
        assert!(yaml.contains("orgSync: true"));
        assert!(yaml.contains("move: true"));
        assert!(yaml.contains("duplicate: false"));
        assert!(yaml.contains("original_item_type"));
        // icon is None → should be omitted
        assert!(!yaml.contains("icon:"));
        // template is None → should be omitted
        assert!(!yaml.contains("template:"));
        // Omit empty collections
        assert!(!yaml.contains("statuses:"));
        assert!(!yaml.contains("defaultStatus:"));
        assert!(!yaml.contains("priorityLevels:"));
    }

    #[test]
    fn test_issue_config_yaml_serialization() {
        let config = default_issue_config(&CentyConfig::default());
        let yaml = serde_yaml::to_string(&config).expect("Should serialize");

        assert!(yaml.contains("name: Issue"));
        assert!(yaml.contains("icon: clipboard"));
        assert!(yaml.contains("identifier: uuid"));
        assert!(yaml.contains("displayNumber: true"));
        assert!(yaml.contains("softDelete: true"));
        assert!(yaml.contains("move: true"));
        assert!(yaml.contains("defaultStatus: open"));
        assert!(yaml.contains("template: template.md"));
    }

    #[test]
    fn test_doc_config_yaml_serialization() {
        let config = default_doc_config();
        let yaml = serde_yaml::to_string(&config).expect("Should serialize");

        assert!(yaml.contains("name: Doc"));
        assert!(yaml.contains("icon: document"));
        assert!(yaml.contains("identifier: slug"));
        assert!(yaml.contains("displayNumber: false"));
        assert!(yaml.contains("softDelete: false"));
        // Docs have no statuses, defaultStatus, or priorityLevels
        assert!(!yaml.contains("statuses"));
        assert!(!yaml.contains("defaultStatus"));
        assert!(!yaml.contains("priorityLevels"));
        // No template for docs
        assert!(!yaml.contains("template:"));
    }

    #[test]
    fn test_item_type_config_yaml_roundtrip() {
        let config = default_issue_config(&CentyConfig::default());
        let yaml = serde_yaml::to_string(&config).expect("Should serialize");
        let deserialized: ItemTypeConfig = serde_yaml::from_str(&yaml).expect("Should deserialize");

        assert_eq!(deserialized.name, "Issue");
        assert_eq!(deserialized.icon, Some("clipboard".to_string()));
        assert_eq!(deserialized.statuses.len(), config.statuses.len());
        assert_eq!(deserialized.default_status, config.default_status);
        assert_eq!(deserialized.priority_levels, config.priority_levels);
        assert_eq!(
            deserialized.features.soft_delete,
            config.features.soft_delete
        );
        assert_eq!(deserialized.template, config.template);
    }

    // ─── Backward-compat deserialization ─────────────────────────────────────

    #[test]
    fn test_legacy_yaml_without_new_fields_deserializes() {
        // A config.yaml from before softDelete/icon/template were added.
        let yaml = "name: Issue\nidentifier: uuid\nfeatures:\n  displayNumber: true\n  status: true\n  priority: true\n  assets: true\n  orgSync: true\n  move: true\n  duplicate: true\nstatuses:\n  - open\n  - closed\ndefaultStatus: open\npriorityLevels: 3\n";
        let config: ItemTypeConfig = serde_yaml::from_str(yaml).expect("Should deserialize");

        assert_eq!(config.name, "Issue");
        // New optional fields default to safe values
        assert!(config.icon.is_none());
        assert!(config.template.is_none());
        assert!(!config.features.soft_delete);
    }

    #[test]
    fn test_yaml_with_icon_and_template() {
        let yaml = "name: Task\nicon: tasks\nidentifier: uuid\nfeatures:\n  displayNumber: true\n  status: true\n  priority: false\n  softDelete: false\n  assets: false\n  orgSync: false\n  move: true\n  duplicate: true\nstatuses:\n  - open\n  - closed\ndefaultStatus: open\ntemplate: task-template.md\n";
        let config: ItemTypeConfig = serde_yaml::from_str(yaml).expect("Should deserialize");

        assert_eq!(config.name, "Task");
        assert_eq!(config.icon, Some("tasks".to_string()));
        assert_eq!(config.template, Some("task-template.md".to_string()));
    }

    #[test]
    fn test_yaml_soft_delete_feature() {
        let yaml = "name: Bug\nidentifier: uuid\nfeatures:\n  displayNumber: false\n  status: true\n  priority: true\n  softDelete: true\n  assets: false\n  orgSync: false\n  move: false\n  duplicate: false\n";
        let config: ItemTypeConfig = serde_yaml::from_str(yaml).expect("Should deserialize");

        assert!(config.features.soft_delete);
    }

    // ─── Validation tests ─────────────────────────────────────────────────────

    #[test]
    fn test_validate_item_type_config_valid() {
        let config = ItemTypeConfig {
            name: "Issue".to_string(),
            icon: None,
            identifier: IdStrategy::Uuid,
            features: ItemTypeFeatures::default(),
            statuses: vec!["open".to_string(), "closed".to_string()],
            default_status: Some("open".to_string()),
            priority_levels: Some(3),
            custom_fields: Vec::new(),
            template: None,
        };
        assert!(validate_item_type_config(&config).is_ok());
    }

    #[test]
    fn test_validate_item_type_config_empty_name() {
        let config = ItemTypeConfig {
            name: String::new(),
            icon: None,
            identifier: IdStrategy::Uuid,
            features: ItemTypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
            template: None,
        };
        let result = validate_item_type_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name must not be empty"));
    }

    #[test]
    fn test_validate_item_type_config_whitespace_name() {
        let config = ItemTypeConfig {
            name: "   ".to_string(),
            icon: None,
            identifier: IdStrategy::Uuid,
            features: ItemTypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
            template: None,
        };
        let result = validate_item_type_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_item_type_config_zero_priority_levels() {
        let config = ItemTypeConfig {
            name: "Issue".to_string(),
            icon: None,
            identifier: IdStrategy::Uuid,
            features: ItemTypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: Some(0),
            custom_fields: Vec::new(),
            template: None,
        };
        let result = validate_item_type_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("priorityLevels must be greater than 0"));
    }

    #[test]
    fn test_validate_item_type_config_none_priority_levels_ok() {
        let config = ItemTypeConfig {
            name: "Doc".to_string(),
            icon: None,
            identifier: IdStrategy::Slug,
            features: ItemTypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
            template: None,
        };
        assert!(validate_item_type_config(&config).is_ok());
    }

    #[test]
    fn test_validate_item_type_config_empty_status_name() {
        let config = ItemTypeConfig {
            name: "Issue".to_string(),
            icon: None,
            identifier: IdStrategy::Uuid,
            features: ItemTypeFeatures::default(),
            statuses: vec!["open".to_string(), String::new()],
            default_status: Some("open".to_string()),
            priority_levels: None,
            custom_fields: Vec::new(),
            template: None,
        };
        let result = validate_item_type_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("status names must not be empty"));
    }

    #[test]
    fn test_validate_item_type_config_whitespace_status_name() {
        let config = ItemTypeConfig {
            name: "Issue".to_string(),
            icon: None,
            identifier: IdStrategy::Uuid,
            features: ItemTypeFeatures::default(),
            statuses: vec!["open".to_string(), "  ".to_string()],
            default_status: Some("open".to_string()),
            priority_levels: None,
            custom_fields: Vec::new(),
            template: None,
        };
        let result = validate_item_type_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_item_type_config_default_status_not_in_statuses() {
        let config = ItemTypeConfig {
            name: "Issue".to_string(),
            icon: None,
            identifier: IdStrategy::Uuid,
            features: ItemTypeFeatures::default(),
            statuses: vec!["open".to_string(), "closed".to_string()],
            default_status: Some("in-progress".to_string()),
            priority_levels: None,
            custom_fields: Vec::new(),
            template: None,
        };
        let err = validate_item_type_config(&config).unwrap_err();
        assert!(err.contains("defaultStatus"));
        assert!(err.contains("in-progress"));
    }

    #[test]
    fn test_validate_item_type_config_no_statuses_no_default_ok() {
        let config = ItemTypeConfig {
            name: "Doc".to_string(),
            icon: None,
            identifier: IdStrategy::Slug,
            features: ItemTypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
            template: None,
        };
        assert!(validate_item_type_config(&config).is_ok());
    }

    // ─── mdstore conversion ───────────────────────────────────────────────────

    #[test]
    fn test_type_config_from_item_type_config() {
        let item_config = default_issue_config(&CentyConfig::default());
        let type_config = TypeConfig::from(&item_config);

        assert_eq!(type_config.name, "Issue");
        assert_eq!(type_config.identifier, IdStrategy::Uuid);
        assert_eq!(type_config.statuses, item_config.statuses);
        assert_eq!(type_config.default_status, item_config.default_status);
        assert_eq!(type_config.priority_levels, item_config.priority_levels);
        assert!(type_config.features.display_number);
        assert!(type_config.features.status);
        assert!(type_config.features.priority);
        assert!(type_config.features.assets);
        assert!(type_config.features.org_sync);
        assert!(type_config.features.move_item);
        assert!(type_config.features.duplicate);
    }

    #[test]
    fn test_type_config_conversion_drops_new_fields() {
        // icon, soft_delete, and template are centy-only and dropped in conversion
        let item_config = ItemTypeConfig {
            name: "Task".to_string(),
            icon: Some("tasks".to_string()),
            identifier: IdStrategy::Uuid,
            features: ItemTypeFeatures {
                soft_delete: true,
                ..ItemTypeFeatures::default()
            },
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
            template: Some("task.md".to_string()),
        };
        let type_config = TypeConfig::from(&item_config);
        // TypeConfig has no icon, soft_delete, or template — conversion is lossy by design
        assert_eq!(type_config.name, "Task");
    }

    // ─── File I/O tests ───────────────────────────────────────────────────────

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
        assert_eq!(read.icon, Some("clipboard".to_string()));
        assert_eq!(read.statuses, config.statuses);
        assert_eq!(read.features.soft_delete, config.features.soft_delete);
        assert_eq!(read.template, config.template);
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
        fs::create_dir_all(centy_dir.join("archived"))
            .await
            .expect("create archived/");

        let config = CentyConfig::default();
        let created = migrate_to_item_type_configs(temp.path(), &config)
            .await
            .expect("Should migrate");

        assert_eq!(created.len(), 3);
        assert!(created.contains(&"issues/config.yaml".to_string()));
        assert!(created.contains(&"docs/config.yaml".to_string()));
        assert!(created.contains(&"archived/config.yaml".to_string()));

        // Files should exist on disk
        assert!(centy_dir.join("issues").join("config.yaml").exists());
        assert!(centy_dir.join("docs").join("config.yaml").exists());
        assert!(centy_dir.join("archived").join("config.yaml").exists());
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
        assert_eq!(issues.icon, Some("clipboard".to_string()));

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
            icon: None,
            identifier: IdStrategy::Uuid,
            features: ItemTypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
            template: None,
        };
        let yaml = serde_yaml::to_string(&config_a).unwrap();
        fs::write(dir_a.join("config.yaml"), &yaml).await.unwrap();

        let dir_b = centy_dir.join("zzz-issues");
        fs::create_dir_all(&dir_b).await.unwrap();
        let config_b = ItemTypeConfig {
            name: "Issue".to_string(),
            icon: None,
            identifier: IdStrategy::Uuid,
            features: ItemTypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
            template: None,
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
    async fn test_registry_resolve_by_folder_case_insensitive() {
        let temp = tempdir().expect("Should create temp dir");
        let centy_dir = temp.path().join(".centy");

        // Create a custom type where name != folder
        let epics_dir = centy_dir.join("my-epics");
        fs::create_dir_all(&epics_dir).await.unwrap();
        let epic_config = ItemTypeConfig {
            name: "Epic".to_string(),
            icon: None,
            identifier: IdStrategy::Uuid,
            features: ItemTypeFeatures::default(),
            statuses: Vec::new(),
            default_status: None,
            priority_levels: None,
            custom_fields: Vec::new(),
            template: None,
        };
        let yaml = serde_yaml::to_string(&epic_config).unwrap();
        fs::write(epics_dir.join("config.yaml"), &yaml)
            .await
            .unwrap();

        let registry = ItemTypeRegistry::build(temp.path()).await.unwrap();

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
        fs::create_dir_all(centy_dir.join("archived"))
            .await
            .expect("create archived/");

        // Pre-create issues/config.yaml
        fs::write(
            centy_dir.join("issues").join("config.yaml"),
            "name: CustomIssue\nidentifier: uuid\nfeatures:\n  displayNumber: false\n  status: false\n  priority: false\n  softDelete: false\n  assets: false\n  orgSync: false\n  move: false\n  duplicate: false\n",
        )
        .await
        .expect("write");

        let config = CentyConfig::default();
        let created = migrate_to_item_type_configs(temp.path(), &config)
            .await
            .expect("Should migrate");

        // Only docs and archived should be created
        assert_eq!(created.len(), 2);
        assert!(created.contains(&"docs/config.yaml".to_string()));
        assert!(created.contains(&"archived/config.yaml".to_string()));

        // Existing file should be untouched
        let content = fs::read_to_string(centy_dir.join("issues").join("config.yaml"))
            .await
            .expect("read");
        assert!(content.contains("name: CustomIssue"));
    }

    #[test]
    fn test_issue_config_custom_fields_mapped() {
        let mut config = CentyConfig::default();
        config.custom_fields = vec![CustomFieldDef {
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
