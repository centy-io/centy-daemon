use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

use crate::utils::get_centy_path;
use super::error::AuthError;
use super::provider::AuthCredentials;

/// Provider-specific authentication config stored in config.local.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskImportAuth {
    /// Map of provider name → credentials
    #[serde(default)]
    pub providers: HashMap<String, AuthCredentials>,
}

/// Read auth config from config.local.json
pub async fn read_auth_config(project_path: &Path) -> Result<TaskImportAuth, AuthError> {
    let config_path = get_centy_path(project_path).join("config.local.json");

    if !config_path.exists() {
        return Ok(TaskImportAuth::default());
    }

    let content = fs::read_to_string(&config_path).await?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    // Extract taskImport.auth section if it exists
    let auth = config
        .get("taskImport")
        .and_then(|ti| ti.get("auth"))
        .and_then(|a| serde_json::from_value(a.clone()).ok())
        .unwrap_or_default();

    Ok(auth)
}

/// Write auth config to config.local.json (merge with existing)
pub async fn write_auth_config(
    project_path: &Path,
    auth: &TaskImportAuth,
) -> Result<(), AuthError> {
    let config_path = get_centy_path(project_path).join("config.local.json");

    // Read existing config
    let mut existing: serde_json::Value = if config_path.exists() {
        let content = fs::read_to_string(&config_path).await?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };

    // Merge auth config
    if existing.get("taskImport").is_none() {
        existing["taskImport"] = serde_json::json!({});
    }
    existing["taskImport"]["auth"] = serde_json::to_value(auth)?;

    // Write back
    let content = serde_json::to_string_pretty(&existing)?;
    fs::write(&config_path, content).await?;

    Ok(())
}

/// Get credentials for a specific provider
pub async fn get_provider_credentials(
    project_path: &Path,
    provider_name: &str,
) -> Result<Option<AuthCredentials>, AuthError> {
    let auth = read_auth_config(project_path).await?;
    Ok(auth.providers.get(provider_name).cloned())
}
