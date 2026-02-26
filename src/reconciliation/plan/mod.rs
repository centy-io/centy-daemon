#![allow(unknown_lints, max_nesting_depth)]
pub(crate) mod helpers;
mod types;
pub use types::{FileInfo, PlanError, ReconciliationPlan};
use helpers::scan_centy_folder;
use super::managed_files::get_managed_files;
use crate::manifest::ManagedFileType;
use crate::utils::{compute_file_hash, compute_hash, get_centy_path};
use std::collections::HashSet;
use std::path::Path;
/// Build a reconciliation plan for the given project path
#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines)]
pub async fn build_reconciliation_plan(project_path: &Path) -> Result<ReconciliationPlan, PlanError> {
    let centy_path = get_centy_path(project_path);
    let managed_templates = get_managed_files();
    let mut plan = ReconciliationPlan::default();
    let files_on_disk = scan_centy_folder(&centy_path);
    let managed_paths: HashSet<String> = managed_templates.keys().cloned().collect();
    for (path, template) in &managed_templates {
        let full_path = centy_path.join(path.trim_end_matches('/'));
        let exists_on_disk = files_on_disk.contains(path);
        let file_info = FileInfo {
            path: path.clone(),
            file_type: template.file_type.clone(),
            hash: template.content.as_ref().map(|c| compute_hash(c)).unwrap_or_default(),
            content_preview: template.content.as_ref().map(|c| c.chars().take(100).collect::<String>()),
        };
        if exists_on_disk {
            match &template.file_type {
                ManagedFileType::Directory => { plan.up_to_date.push(file_info); }
                ManagedFileType::File => {
                    if let Some(expected_content) = &template.content {
                        let expected_hash = compute_hash(expected_content);
                        let actual_hash = compute_file_hash(&full_path).await.unwrap_or_default();
                        if actual_hash == expected_hash { plan.up_to_date.push(file_info); }
                        else { plan.to_reset.push(FileInfo { hash: actual_hash, ..file_info }); }
                    } else { plan.up_to_date.push(file_info); }
                }
            }
        } else { plan.to_create.push(file_info); }
    }
    for disk_path in &files_on_disk {
        if !managed_paths.contains(disk_path) {
            let full_path = centy_path.join(disk_path.trim_end_matches('/'));
            let is_dir = full_path.is_dir();
            let hash = if is_dir { String::new() } else { compute_file_hash(&full_path).await.unwrap_or_default() };
            plan.user_files.push(FileInfo {
                path: disk_path.clone(),
                file_type: if is_dir { ManagedFileType::Directory } else { ManagedFileType::File },
                hash,
                content_preview: None,
            });
        }
    }
    Ok(plan)
}
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
#[path = "../plan_tests.rs"]
mod tests;
#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
#[path = "../plan_tests_2.rs"]
mod tests2;
