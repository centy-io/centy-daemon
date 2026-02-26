use super::migrate;
use super::types::{CentyConfig, ProjectMetadata};
use crate::utils::get_centy_path;
use std::path::Path;
use tokio::fs;
use tracing::warn;
/// Read the configuration file.
pub async fn read_config(project_path: &Path) -> Result<Option<CentyConfig>, mdstore::ConfigError> {
    let config_path = get_centy_path(project_path).join("config.json");
    if !config_path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&config_path).await?;
    let mut raw: serde_json::Value = serde_json::from_str(&content)?;
    let mut needs_write = false;
    if migrate::needs_migration(&raw) {
        warn!("Detected deprecated nested config format in {}, auto-converting to flat dot-separated keys", config_path.display());
        raw = migrate::flatten_config(raw);
        needs_write = true;
    }
    if !raw.as_object().is_some_and(|o| o.contains_key("hooks")) {
        needs_write = true;
    }
    if let Some(obj) = raw.as_object_mut() {
        if obj.remove("defaultState").is_some() {
            needs_write = true;
        }
    }
    if raw
        .as_object()
        .is_some_and(|o| o.contains_key("allowedStates"))
    {
        needs_write = true;
    }
    let nested = migrate::unflatten_config(raw);
    let config: CentyConfig = serde_json::from_value(nested)?;
    if needs_write {
        write_config(project_path, &config).await?;
    }
    Ok(Some(config))
}
/// Write the configuration file in flat dot-separated format.
pub async fn write_config(
    project_path: &Path,
    config: &CentyConfig,
) -> Result<(), mdstore::ConfigError> {
    let config_path = get_centy_path(project_path).join("config.json");
    let nested_value = serde_json::to_value(config)?;
    let flat_value = migrate::flatten_config(nested_value);
    let content = serde_json::to_string_pretty(&flat_value)?;
    fs::write(&config_path, content).await?;
    Ok(())
}
/// Read the project metadata file (.centy/project.json)
pub async fn read_project_metadata(
    project_path: &Path,
) -> Result<Option<ProjectMetadata>, mdstore::ConfigError> {
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
) -> Result<(), mdstore::ConfigError> {
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
) -> Result<(), mdstore::ConfigError> {
    let mut metadata = read_project_metadata(project_path)
        .await?
        .unwrap_or_default();
    metadata.title = title;
    write_project_metadata(project_path, &metadata).await
}
