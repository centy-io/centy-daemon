//! Migration helpers for item type configs.
use super::convert::{default_archived_config, default_issue_config};
use super::defaults::default_doc_config;
use super::io::{read_item_type_config, write_item_type_config};
use crate::config::CentyConfig;
use std::path::Path;
/// Re-write every existing `config.yaml` to remove the now-removed `status` feature flag.
///
/// The field is deserialized as an unknown key (silently ignored), so this is purely a
/// cosmetic clean-up. Configs that cannot be parsed are skipped without error.
pub async fn migrate_strip_status_feature(project_path: &Path) -> Result<(), mdstore::ConfigError> {
    let centy_path = crate::utils::get_centy_path(project_path);
    if !centy_path.exists() {
        return Ok(());
    }
    let mut entries = tokio::fs::read_dir(&centy_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_dir() {
            continue;
        }
        let folder = entry.file_name().to_string_lossy().to_string();
        // Skip configs that fail to parse — they may be custom/partial YAML
        if let Ok(Some(config)) = read_item_type_config(project_path, &folder).await {
            write_item_type_config(project_path, &folder, &config).await?;
        }
    }
    Ok(())
}
/// Create `config.yaml` for issues, docs, and archived if they don't already exist.
///
/// Returns the list of relative paths that were created.
/// `legacy_statuses` is read from `allowedStates` in `config.json` before migration.
pub async fn migrate_to_item_type_configs(
    project_path: &Path,
    config: &CentyConfig,
    legacy_statuses: Option<Vec<String>>,
) -> Result<Vec<String>, mdstore::ConfigError> {
    let mut created = Vec::new();
    let centy_path = crate::utils::get_centy_path(project_path);
    // Issues
    let issues_config_path = centy_path.join("issues").join("config.yaml");
    if !issues_config_path.exists() {
        let mut issue_config = default_issue_config(config);
        if let Some(statuses) = legacy_statuses {
            issue_config.statuses = statuses;
        }
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
