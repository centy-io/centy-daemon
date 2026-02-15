pub mod item_type_config;
pub mod migrate;

use crate::hooks::HookDefinition;
use crate::link::CustomLinkTypeDefinition;
use crate::utils::get_centy_path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tokio::fs;
use tracing::warn;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

/// Custom field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomFieldDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
}

/// Default priority levels (3 = high/medium/low)
fn default_priority_levels() -> u32 {
    3
}

/// Default allowed states for issues
fn default_allowed_states() -> Vec<String> {
    vec![
        "open".to_string(),
        "planning".to_string(),
        "in-progress".to_string(),
        "closed".to_string(),
    ]
}

/// Workspace configuration section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceConfig {
    /// Whether opening a temp workspace automatically updates the issue status to "in-progress".
    /// `None` means the user hasn't configured it yet (server returns `requires_status_config`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_status_on_open: Option<bool>,
}

/// Centy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CentyConfig {
    /// Project version (semver string). Defaults to daemon version if not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Number of priority levels (1-10). Default is 3 (high/medium/low).
    /// - 2 levels: high, low
    /// - 3 levels: high, medium, low
    /// - 4 levels: critical, high, medium, low
    /// - 5+ levels: P1, P2, P3, etc.
    #[serde(default = "default_priority_levels")]
    pub priority_levels: u32,
    #[serde(default)]
    pub custom_fields: Vec<CustomFieldDefinition>,
    #[serde(default)]
    pub defaults: HashMap<String, String>,
    /// Allowed status values for issues (default: `["open", "planning", "in-progress", "closed"]`)
    #[serde(default = "default_allowed_states")]
    pub allowed_states: Vec<String>,
    /// State colors: state name → hex color (e.g., "open" → "#10b981")
    #[serde(default)]
    pub state_colors: HashMap<String, String>,
    /// Priority colors: priority level → hex color (e.g., "1" → "#ef4444")
    #[serde(default)]
    pub priority_colors: HashMap<String, String>,
    /// Custom link types (in addition to built-in types)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_link_types: Vec<CustomLinkTypeDefinition>,
    /// Default editor ID for this project (e.g., "vscode", "terminal", "zed").
    /// Overrides the user-level default. Empty means use user default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_editor: Option<String>,
    /// Lifecycle hooks (bash scripts to run before/after operations)
    #[serde(default)]
    pub hooks: Vec<HookDefinition>,
    /// Workspace settings (e.g. auto-update issue status on open)
    #[serde(default)]
    pub workspace: WorkspaceConfig,
}

impl CentyConfig {
    /// Get the effective version (config version or daemon default).
    #[must_use]
    pub fn effective_version(&self) -> String {
        self.version
            .clone()
            .unwrap_or_else(|| crate::utils::CENTY_VERSION.to_string())
    }
}

impl Default for CentyConfig {
    fn default() -> Self {
        Self {
            version: None,
            priority_levels: default_priority_levels(),
            custom_fields: Vec::new(),
            defaults: HashMap::new(),
            allowed_states: default_allowed_states(),
            state_colors: HashMap::new(),
            priority_colors: HashMap::new(),
            custom_link_types: Vec::new(),
            default_editor: None,
            hooks: Vec::new(),
            workspace: WorkspaceConfig::default(),
        }
    }
}

/// Read the configuration file.
/// Automatically migrates the deprecated nested config format to flat dot-separated keys.
/// Also normalizes by adding missing sections (e.g. `hooks`) that were introduced
/// after the project was created, then writes back to disk.
pub async fn read_config(project_path: &Path) -> Result<Option<CentyConfig>, ConfigError> {
    let config_path = get_centy_path(project_path).join("config.json");

    if !config_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&config_path).await?;
    let mut raw: serde_json::Value = serde_json::from_str(&content)?;

    // Migrate nested format to flat dot-separated keys if needed
    let mut needs_write = false;
    if migrate::needs_migration(&raw) {
        warn!(
            "Detected deprecated nested config format in {}, auto-converting to flat dot-separated keys",
            config_path.display()
        );
        raw = migrate::flatten_config(raw);
        needs_write = true;
    }

    // Check if normalization is needed (e.g. missing "hooks" key)
    if !raw.as_object().is_some_and(|o| o.contains_key("hooks")) {
        needs_write = true;
    }

    // Remove deprecated defaultState (now lives in per-item-type config.yaml)
    if let Some(obj) = raw.as_object_mut() {
        if obj.remove("defaultState").is_some() {
            needs_write = true;
        }
    }

    // Unflatten for serde deserialization (serde expects nested objects)
    let nested = migrate::unflatten_config(raw);
    let config: CentyConfig = serde_json::from_value(nested)?;

    // Write back if migration or normalization occurred
    if needs_write {
        write_config(project_path, &config).await?;
    }

    Ok(Some(config))
}

/// Write the configuration file in flat dot-separated format.
pub async fn write_config(project_path: &Path, config: &CentyConfig) -> Result<(), ConfigError> {
    let config_path = get_centy_path(project_path).join("config.json");
    let nested_value = serde_json::to_value(config)?;
    let flat_value = migrate::flatten_config(nested_value);
    let content = serde_json::to_string_pretty(&flat_value)?;
    fs::write(&config_path, content).await?;
    Ok(())
}

/// Project metadata stored in .centy/project.json (version-controlled, shared with team)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    /// Project-scope custom title (visible to all users)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Read the project metadata file (.centy/project.json)
pub async fn read_project_metadata(
    project_path: &Path,
) -> Result<Option<ProjectMetadata>, ConfigError> {
    let metadata_path = get_centy_path(project_path).join("project.json");

    if !metadata_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&metadata_path).await?;
    let metadata: ProjectMetadata = serde_json::from_str(&content)?;
    Ok(Some(metadata))
}

/// Write the project metadata file (.centy/project.json)
pub async fn write_project_metadata(
    project_path: &Path,
    metadata: &ProjectMetadata,
) -> Result<(), ConfigError> {
    let metadata_path = get_centy_path(project_path).join("project.json");
    let content = serde_json::to_string_pretty(metadata)?;
    fs::write(&metadata_path, content).await?;
    Ok(())
}

/// Get the project-scope title from .centy/project.json
pub async fn get_project_title(project_path: &Path) -> Option<String> {
    read_project_metadata(project_path)
        .await
        .ok()
        .flatten()
        .and_then(|m| m.title)
}

/// Set the project-scope title in .centy/project.json
pub async fn set_project_title(
    project_path: &Path,
    title: Option<String>,
) -> Result<(), ConfigError> {
    let mut metadata = read_project_metadata(project_path)
        .await?
        .unwrap_or_default();
    metadata.title = title;
    write_project_metadata(project_path, &metadata).await
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_default_priority_levels() {
        assert_eq!(default_priority_levels(), 3);
    }

    #[test]
    fn test_default_allowed_states() {
        let states = default_allowed_states();
        assert_eq!(states.len(), 4);
        assert!(states.contains(&"open".to_string()));
        assert!(states.contains(&"planning".to_string()));
        assert!(states.contains(&"in-progress".to_string()));
        assert!(states.contains(&"closed".to_string()));
    }

    #[test]
    fn test_centy_config_default() {
        let config = CentyConfig::default();

        assert!(config.version.is_none());
        assert_eq!(config.priority_levels, 3);
        assert!(config.custom_fields.is_empty());
        assert!(config.defaults.is_empty());
        assert_eq!(config.allowed_states.len(), 4);
        assert!(config.state_colors.is_empty());
        assert!(config.priority_colors.is_empty());
        assert!(config.custom_link_types.is_empty());
        assert!(config.hooks.is_empty());
        assert!(config.workspace.update_status_on_open.is_none());
    }

    #[test]
    fn test_centy_config_effective_version_with_version() {
        let mut config = CentyConfig::default();
        config.version = Some("1.0.0".to_string());

        assert_eq!(config.effective_version(), "1.0.0");
    }

    #[test]
    fn test_centy_config_effective_version_without_version() {
        let config = CentyConfig::default();

        // Should return the daemon version when no version is set
        assert_eq!(config.effective_version(), crate::utils::CENTY_VERSION);
    }

    #[test]
    fn test_centy_config_serialization_deserialization() {
        let mut config = CentyConfig::default();
        config.version = Some("1.2.3".to_string());
        config.priority_levels = 5;

        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: CentyConfig = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.version, Some("1.2.3".to_string()));
        assert_eq!(deserialized.priority_levels, 5);
    }

    #[test]
    fn test_centy_config_json_uses_camel_case() {
        let config = CentyConfig::default();
        let json = serde_json::to_string(&config).expect("Should serialize");

        // Check for camelCase keys
        assert!(json.contains("priorityLevels"));
        assert!(json.contains("customFields"));
        assert!(json.contains("allowedStates"));
        assert!(json.contains("stateColors"));
        assert!(json.contains("priorityColors"));

        // Should NOT contain snake_case
        assert!(!json.contains("priority_levels"));
        assert!(!json.contains("custom_fields"));
        assert!(!json.contains("allowed_states"));
    }

    #[test]
    fn test_custom_field_definition_serialization() {
        let field = CustomFieldDefinition {
            name: "environment".to_string(),
            field_type: "enum".to_string(),
            required: true,
            default_value: Some("dev".to_string()),
            enum_values: vec!["dev".to_string(), "staging".to_string(), "prod".to_string()],
        };

        let json = serde_json::to_string(&field).expect("Should serialize");
        let deserialized: CustomFieldDefinition =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.name, "environment");
        assert_eq!(deserialized.field_type, "enum");
        assert!(deserialized.required);
        assert_eq!(deserialized.default_value, Some("dev".to_string()));
        assert_eq!(deserialized.enum_values.len(), 3);
    }

    #[test]
    fn test_custom_field_definition_json_uses_camel_case() {
        let field = CustomFieldDefinition {
            name: "test".to_string(),
            field_type: "string".to_string(),
            required: false,
            default_value: None,
            enum_values: vec![],
        };

        let json = serde_json::to_string(&field).expect("Should serialize");

        // Check for camelCase keys (type becomes "type" due to rename attribute)
        assert!(json.contains("\"type\""));
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"required\""));

        // Should NOT contain snake_case for field_type
        assert!(!json.contains("field_type"));
    }

    #[test]
    fn test_project_metadata_default() {
        let metadata = ProjectMetadata::default();
        assert!(metadata.title.is_none());
    }

    #[test]
    fn test_project_metadata_serialization() {
        let mut metadata = ProjectMetadata::default();
        metadata.title = Some("My Project".to_string());

        let json = serde_json::to_string(&metadata).expect("Should serialize");
        let deserialized: ProjectMetadata =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.title, Some("My Project".to_string()));
    }

    #[tokio::test]
    async fn test_read_config_nonexistent_returns_none() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let result = read_config(temp_dir.path())
            .await
            .expect("Should not error");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_read_write_config_roundtrip() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_dir = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_dir)
            .await
            .expect("Should create .centy dir");

        let mut config = CentyConfig::default();
        config.version = Some("2.0.0".to_string());
        config.priority_levels = 4;

        write_config(temp_dir.path(), &config)
            .await
            .expect("Should write");
        let read_config = read_config(temp_dir.path())
            .await
            .expect("Should read")
            .expect("Config should exist");

        assert_eq!(read_config.version, Some("2.0.0".to_string()));
        assert_eq!(read_config.priority_levels, 4);
    }

    #[tokio::test]
    async fn test_read_project_metadata_nonexistent_returns_none() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let result = read_project_metadata(temp_dir.path())
            .await
            .expect("Should not error");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_read_write_project_metadata_roundtrip() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_dir = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_dir)
            .await
            .expect("Should create .centy dir");

        let mut metadata = ProjectMetadata::default();
        metadata.title = Some("Test Project".to_string());

        write_project_metadata(temp_dir.path(), &metadata)
            .await
            .expect("Should write");
        let read_metadata = read_project_metadata(temp_dir.path())
            .await
            .expect("Should read")
            .expect("Metadata should exist");

        assert_eq!(read_metadata.title, Some("Test Project".to_string()));
    }

    #[tokio::test]
    async fn test_get_project_title_nonexistent() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let title = get_project_title(temp_dir.path()).await;

        assert!(title.is_none());
    }

    #[tokio::test]
    async fn test_set_and_get_project_title() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_dir = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_dir)
            .await
            .expect("Should create .centy dir");

        // Set title
        set_project_title(temp_dir.path(), Some("My Awesome Project".to_string()))
            .await
            .expect("Should set title");

        // Get title
        let title = get_project_title(temp_dir.path()).await;
        assert_eq!(title, Some("My Awesome Project".to_string()));

        // Clear title
        set_project_title(temp_dir.path(), None)
            .await
            .expect("Should clear title");

        let title = get_project_title(temp_dir.path()).await;
        assert!(title.is_none());
    }

    #[tokio::test]
    async fn test_read_config_normalizes_missing_hooks() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_dir = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_dir)
            .await
            .expect("Should create .centy dir");

        // Write a config.json in flat format WITHOUT the hooks key
        let config_without_hooks = r#"{
  "priorityLevels": 3,
  "customFields": [],
  "defaults": {},
  "allowedStates": ["open", "planning", "in-progress", "closed"],
  "defaultState": "open",
  "stateColors": {},
  "priorityColors": {}
}"#;
        let config_path = centy_dir.join("config.json");
        fs::write(&config_path, config_without_hooks)
            .await
            .expect("Should write config");

        // Reading should succeed and return empty hooks
        let config = read_config(temp_dir.path())
            .await
            .expect("Should read")
            .expect("Config should exist");
        assert!(config.hooks.is_empty());

        // The file should now contain the hooks key (in flat format)
        let raw = fs::read_to_string(&config_path)
            .await
            .expect("Should read file");
        let value: serde_json::Value = serde_json::from_str(&raw).expect("Should parse");
        assert!(
            value.as_object().unwrap().contains_key("hooks"),
            "config.json should now contain the hooks key"
        );
        // Should not contain removed llm config
        assert!(
            !value.as_object().unwrap().contains_key("llm"),
            "config.json should not contain llm config"
        );
    }

    #[tokio::test]
    async fn test_read_config_does_not_rewrite_when_hooks_present_flat_format() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_dir = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_dir)
            .await
            .expect("Should create .centy dir");

        // Write a config.json in flat format WITH the hooks key (no deprecated fields)
        let config_with_hooks = r#"{
  "allowedStates": [
    "open",
    "planning",
    "in-progress",
    "closed"
  ],
  "customFields": [],
  "defaults": {},
  "hooks": [],
  "priorityColors": {},
  "priorityLevels": 3,
  "stateColors": {}
}"#;
        let config_path = centy_dir.join("config.json");
        fs::write(&config_path, config_with_hooks)
            .await
            .expect("Should write config");

        // Read config
        let config = read_config(temp_dir.path())
            .await
            .expect("Should read")
            .expect("Config should exist");
        assert!(config.hooks.is_empty());

        // File content should remain unchanged (not rewritten)
        let raw = fs::read_to_string(&config_path)
            .await
            .expect("Should read file");
        assert_eq!(raw, config_with_hooks);
    }

    #[tokio::test]
    async fn test_read_config_flat_format_works() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_dir = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_dir)
            .await
            .expect("Should create .centy dir");

        // Write a config.json already in flat format
        let flat_config = r#"{
  "version": "0.0.1",
  "priorityLevels": 5,
  "customFields": [],
  "defaults": {},
  "allowedStates": ["open", "closed"],
  "defaultState": "open",
  "stateColors": {},
  "priorityColors": {},
  "hooks": []
}"#;
        let config_path = centy_dir.join("config.json");
        fs::write(&config_path, flat_config)
            .await
            .expect("Should write config");

        let config = read_config(temp_dir.path())
            .await
            .expect("Should read")
            .expect("Config should exist");

        assert_eq!(config.version, Some("0.0.1".to_string()));
        assert_eq!(config.priority_levels, 5);
    }

    #[test]
    fn test_hooks_always_serialized_even_when_empty() {
        let config = CentyConfig::default();
        let json = serde_json::to_string(&config).expect("Should serialize");
        assert!(
            json.contains("\"hooks\""),
            "hooks key should be present in serialized JSON even when empty"
        );
    }

    #[test]
    fn test_workspace_config_default() {
        let ws = WorkspaceConfig::default();
        assert!(ws.update_status_on_open.is_none());
    }

    #[test]
    fn test_workspace_config_serialization_skips_none() {
        let ws = WorkspaceConfig::default();
        let json = serde_json::to_string(&ws).expect("Should serialize");
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_workspace_config_serialization_with_value() {
        let ws = WorkspaceConfig {
            update_status_on_open: Some(true),
        };
        let json = serde_json::to_string(&ws).expect("Should serialize");
        assert!(json.contains("updateStatusOnOpen"));
        let deserialized: WorkspaceConfig =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.update_status_on_open, Some(true));
    }

    #[test]
    fn test_centy_config_workspace_roundtrip() {
        let mut config = CentyConfig::default();
        config.workspace.update_status_on_open = Some(false);

        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: CentyConfig = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.workspace.update_status_on_open, Some(false));
    }
}
