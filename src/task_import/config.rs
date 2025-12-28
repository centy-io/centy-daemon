use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

use crate::utils::get_centy_path;
use super::error::ConfigError;

/// Import configuration stored in .centy/config.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskImportConfig {
    /// Map of provider name → provider settings
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

/// Configuration for a specific provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    /// Provider name (e.g., "github", "gitlab")
    pub provider: String,

    /// Source identifier (e.g., "owner/repo" for GitHub)
    pub source_id: String,

    /// Field mappings for this provider
    #[serde(default)]
    pub field_mappings: FieldMappings,

    /// Provider-specific settings
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

/// Field mappings from provider to Centy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FieldMappings {
    /// Map external labels to Centy custom fields
    /// Example: { "bug": "type=bug", "priority:high": "priority=1" }
    #[serde(default)]
    pub label_to_custom_field: HashMap<String, String>,

    /// Map external status to Centy status
    /// Example: { "open": "open", "closed": "closed" }
    #[serde(default)]
    pub status_mapping: HashMap<String, String>,

    /// Default Centy status if external status not in mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_status: Option<String>,

    /// Default Centy priority if not specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_priority: Option<u32>,
}

/// Read task import configuration from .centy/config.json
pub async fn read_config(project_path: &Path) -> Result<TaskImportConfig, ConfigError> {
    let config_path = get_centy_path(project_path).join("config.json");

    if !config_path.exists() {
        return Ok(TaskImportConfig::default());
    }

    let content = fs::read_to_string(&config_path).await?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    // Extract taskImport section if it exists
    let task_import = config
        .get("taskImport")
        .and_then(|ti| serde_json::from_value(ti.clone()).ok())
        .unwrap_or_default();

    Ok(task_import)
}

/// Write task import configuration to .centy/config.json (merge with existing)
pub async fn write_config(
    project_path: &Path,
    task_import_config: &TaskImportConfig,
) -> Result<(), ConfigError> {
    let config_path = get_centy_path(project_path).join("config.json");

    // Read existing config
    let mut existing: serde_json::Value = if config_path.exists() {
        let content = fs::read_to_string(&config_path).await?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    // Merge task import config
    existing["taskImport"] = serde_json::to_value(task_import_config)?;

    // Write back
    let content = serde_json::to_string_pretty(&existing)?;
    fs::write(&config_path, content).await?;

    Ok(())
}

/// Get a specific provider configuration
pub async fn get_provider_config(
    project_path: &Path,
    provider_name: &str,
) -> Result<Option<ProviderConfig>, ConfigError> {
    let config = read_config(project_path).await?;
    Ok(config.providers.get(provider_name).cloned())
}

/// Update a specific provider configuration
pub async fn update_provider_config(
    project_path: &Path,
    provider_name: &str,
    provider_config: ProviderConfig,
) -> Result<(), ConfigError> {
    let mut config = read_config(project_path).await?;
    config
        .providers
        .insert(provider_name.to_string(), provider_config);
    write_config(project_path, &config).await
}
