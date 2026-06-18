use super::managed_files::process_managed_files;
use super::types::{PlanError, ReconciliationPlan};
use super::user_files::{collect_user_files, scan_centy_folder};
use crate::reconciliation::managed_files::get_managed_files;
use crate::utils::get_centy_path;
use std::collections::HashSet;
use std::path::Path;

/// Build a reconciliation plan for the given project path
pub async fn build_reconciliation_plan(
    project_path: &Path,
) -> Result<ReconciliationPlan, PlanError> {
    let centy_path = get_centy_path(project_path);
    let managed_templates = get_managed_files();
    let mut plan = ReconciliationPlan::default();
    let files_on_disk = scan_centy_folder(&centy_path);
    let managed_paths: HashSet<String> = managed_templates.keys().cloned().collect();
    process_managed_files(&mut plan, &managed_templates, &files_on_disk, &centy_path).await;
    collect_user_files(&mut plan, &files_on_disk, &managed_paths, &centy_path).await;
    Ok(plan)
}
