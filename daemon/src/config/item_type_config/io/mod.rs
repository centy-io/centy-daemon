mod discover;
mod legacy;

pub use discover::{discover_item_types, discover_item_types_map};
pub use legacy::read_legacy_allowed_states;

use super::types::ItemTypeConfig;
use crate::utils::{get_centy_path, with_yaml_header};
use std::path::Path;
use tokio::fs;

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
    let content = with_yaml_header(&serde_yaml::to_string(config)?);
    fs::write(&config_path, content).await?;
    Ok(())
}
