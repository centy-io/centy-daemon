use crate::utils::get_centy_path;
use std::path::Path;
use tokio::fs;

/// Read legacy `allowedStates` from a raw `config.json` file, if present.
/// Returns `None` when config.json is absent, malformed, or has no `allowedStates` key.
/// Must be called **before** `read_config` so the key is still present on disk.
pub async fn read_legacy_allowed_states(project_path: &Path) -> Option<Vec<String>> {
    let config_path = get_centy_path(project_path).join("config.json");
    let content = fs::read_to_string(&config_path).await.ok()?;
    let raw: serde_json::Value = serde_json::from_str(&content).ok()?;
    let arr = raw.get("allowedStates")?.as_array()?;
    let states: Vec<String> = arr
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    if states.is_empty() {
        None
    } else {
        Some(states)
    }
}
