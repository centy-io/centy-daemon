#![allow(unknown_lints, max_nesting_depth)]
mod file_ops;
mod types;
use super::managed_files::get_managed_files;
use super::plan::build_reconciliation_plan;
use crate::config::item_type_config::migrate_to_item_type_configs;
use crate::config::{read_config, write_config, CentyConfig};
use crate::manifest::{create_manifest, read_manifest, update_manifest, write_manifest};
use crate::utils::get_centy_path;
use file_ops::{create_file, merge_file};
use std::path::Path;
use tokio::fs;
pub use types::{ExecuteError, ReconciliationDecisions, ReconciliationResult};
/// Execute the reconciliation plan
#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines)]
pub async fn execute_reconciliation(
    project_path: &Path,
    decisions: ReconciliationDecisions,
    force: bool,
) -> Result<ReconciliationResult, ExecuteError> {
    let centy_path = get_centy_path(project_path);
    let managed_templates = get_managed_files();
    if !centy_path.exists() {
        fs::create_dir_all(&centy_path).await?;
    }
    let mut manifest = read_manifest(project_path)
        .await?
        .unwrap_or_else(create_manifest);
    let plan = build_reconciliation_plan(project_path).await?;
    let mut result = ReconciliationResult::default();
    for file_info in &plan.to_create {
        create_file(&centy_path, &file_info.path, &managed_templates).await?;
        result.created.push(file_info.path.clone());
    }
    for file_info in &plan.to_restore {
        if force || decisions.restore.contains(&file_info.path) {
            create_file(&centy_path, &file_info.path, &managed_templates).await?;
            result.restored.push(file_info.path.clone());
        } else {
            result.skipped.push(file_info.path.clone());
        }
    }
    for file_info in &plan.to_reset {
        let template = managed_templates.get(&file_info.path);
        let has_merge = template
            .map(|t| t.merge_strategy.is_some())
            .unwrap_or(false);
        if has_merge {
            merge_file(&centy_path, &file_info.path, &managed_templates).await?;
            result.reset.push(file_info.path.clone());
        } else if decisions.reset.contains(&file_info.path) {
            create_file(&centy_path, &file_info.path, &managed_templates).await?;
            result.reset.push(file_info.path.clone());
        } else {
            result.skipped.push(file_info.path.clone());
        }
    }
    let config_path = centy_path.join("config.json");
    if !config_path.exists() {
        let default_config = CentyConfig::default();
        write_config(project_path, &default_config).await?;
        result.created.push("config.json".to_string());
    }
    let config = read_config(project_path).await?.unwrap_or_default();
    let migrated = migrate_to_item_type_configs(project_path, &config).await?;
    for path in migrated {
        result.created.push(path);
    }
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;
    result.manifest = manifest;
    Ok(result)
}
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
#[path = "../execute_tests.rs"]
mod tests;
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
#[path = "../execute_tests_2.rs"]
mod tests2;
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
#[path = "../execute_tests_3.rs"]
mod tests3;
