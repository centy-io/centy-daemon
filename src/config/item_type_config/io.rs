use super::types::ItemTypeConfig;
use crate::utils::get_centy_path;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::error;

/// Scan `.centy/*/config.yaml` and return a map of folder → `ItemTypeConfig`.
///
/// Malformed YAML files are logged and skipped; the function does not fail.
#[allow(unknown_lints, max_nesting_depth)]
pub async fn discover_item_types_map(
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
/// Malformed configs are logged and skipped.
pub async fn discover_item_types(
    project_path: &Path,
) -> Result<Vec<ItemTypeConfig>, mdstore::ConfigError> {
    let centy_path = get_centy_path(project_path);
    Ok(discover_item_types_map(&centy_path)
        .await?
        .into_values()
        .collect())
}
