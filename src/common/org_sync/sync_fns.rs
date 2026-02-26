use super::trait_def::OrgSyncable;
use super::types::OrgSyncResult;
use crate::registry::get_org_projects;
use std::path::Path;
/// Orchestrate syncing an org item to all organization projects.
pub async fn sync_to_org_projects<T: OrgSyncable>(
    item: &T,
    source_project_path: &Path,
) -> Vec<OrgSyncResult> {
    let org_slug = match item.org_slug() {
        Some(slug) => slug,
        None => return Vec::new(),
    };
    let source_path_str = source_project_path.to_string_lossy().to_string();
    let org_projects = match get_org_projects(org_slug, Some(&source_path_str)).await {
        Ok(projects) => projects,
        Err(e) => {
            return vec![OrgSyncResult {
                project_path: "<registry>".to_string(),
                success: false,
                error: Some(format!("Failed to get org projects: {e}")),
            }]
        }
    };
    let mut results = Vec::new();
    for project in org_projects {
        let target_path = Path::new(&project.path);
        let result = item.sync_to_project(target_path, org_slug).await;
        results.push(OrgSyncResult {
            project_path: project.path.clone(),
            success: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
        });
    }
    results
}
/// Orchestrate syncing an org item update to all organization projects.
pub async fn sync_update_to_org_projects<T: OrgSyncable>(
    item: &T,
    source_project_path: &Path,
    old_id: Option<&str>,
) -> Vec<OrgSyncResult> {
    let org_slug = match item.org_slug() {
        Some(slug) => slug,
        None => return Vec::new(),
    };
    let source_path_str = source_project_path.to_string_lossy().to_string();
    let org_projects = match get_org_projects(org_slug, Some(&source_path_str)).await {
        Ok(projects) => projects,
        Err(e) => {
            return vec![OrgSyncResult {
                project_path: "<registry>".to_string(),
                success: false,
                error: Some(format!("Failed to get org projects: {e}")),
            }]
        }
    };
    let mut results = Vec::new();
    for project in org_projects {
        let target_path = Path::new(&project.path);
        let result = item
            .sync_update_to_project(target_path, org_slug, old_id)
            .await;
        results.push(OrgSyncResult {
            project_path: project.path.clone(),
            success: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
        });
    }
    results
}
