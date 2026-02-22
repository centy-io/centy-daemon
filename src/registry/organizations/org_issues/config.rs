//! Org-level configuration stored at ~/.centy/orgs/{slug}/config.json

use super::paths::get_org_config_path;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum OrgConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Path error: {0}")]
    PathError(#[from] super::paths::PathError),
}

/// Organization-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgConfig {
    /// Number of priority levels (default: 3)
    #[serde(default = "default_priority_levels")]
    pub priority_levels: u32,
    /// Custom field definitions for org issues
    #[serde(default)]
    pub custom_fields: Vec<OrgCustomFieldDef>,
}

fn default_priority_levels() -> u32 {
    3
}

impl Default for OrgConfig {
    fn default() -> Self {
        Self {
            priority_levels: 3,
            custom_fields: Vec::new(),
        }
    }
}

/// Custom field definition for org config
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgCustomFieldDef {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Read org config, returning defaults if not found
pub async fn get_org_config(org_slug: &str) -> Result<OrgConfig, OrgConfigError> {
    let path = get_org_config_path(org_slug)?;

    if !path.exists() {
        return Ok(OrgConfig::default());
    }

    let content = fs::read_to_string(&path).await?;
    let config: OrgConfig = serde_json::from_str(&content)?;
    Ok(config)
}

/// Write org config
pub async fn update_org_config(org_slug: &str, config: &OrgConfig) -> Result<(), OrgConfigError> {
    let path = get_org_config_path(org_slug)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let temp_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&temp_path, &content).await?;
    fs::rename(&temp_path, &path).await?;

    Ok(())
}
