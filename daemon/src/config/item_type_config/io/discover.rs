use crate::config::item_type_config::ItemTypeConfig;
use crate::utils::get_centy_path;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::error;

/// Try to load an `ItemTypeConfig` from a single directory entry.
/// Returns `None` if the entry is not a directory, has no `config.yaml`, or is malformed.
/// Malformed or unreadable configs are logged before returning `None`.
async fn load_item_type_entry(entry: &fs::DirEntry) -> Option<(String, ItemTypeConfig)> {
    if !entry.file_type().await.ok()?.is_dir() {
        return None;
    }
    let config_path = entry.path().join("config.yaml");
    if !config_path.exists() {
        return None;
    }
    let folder_name = entry.file_name().to_string_lossy().to_string();
    let content = match fs::read_to_string(&config_path).await {
        Ok(c) => c,
        Err(e) => {
            error!(folder = %folder_name, error = %e, "Failed to read config.yaml, skipping type");
            return None;
        }
    };
    match serde_yaml::from_str::<ItemTypeConfig>(&content) {
        Ok(config) => Some((folder_name, config)),
        Err(e) => {
            error!(folder = %folder_name, error = %e, "Malformed config.yaml, skipping type");
            None
        }
    }
}

/// Scan `.centy/*/config.yaml` and return a map of folder → `ItemTypeConfig`.
///
/// Malformed YAML files are logged and skipped; the function does not fail.
pub async fn discover_item_types_map(
    centy_path: &Path,
) -> Result<HashMap<String, ItemTypeConfig>, mdstore::ConfigError> {
    if !centy_path.exists() {
        return Ok(HashMap::new());
    }
    let mut configs = HashMap::new();
    let mut entries = fs::read_dir(centy_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if let Some((folder_name, config)) = load_item_type_entry(&entry).await {
            configs.insert(folder_name, config);
        }
    }
    Ok(configs)
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
