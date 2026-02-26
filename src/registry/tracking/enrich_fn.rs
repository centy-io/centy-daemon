use super::super::types::{ProjectInfo, TrackedProject};
use super::counts::{count_issues, count_md_files};
use crate::config::get_project_title;
use crate::manifest::read_manifest;
use crate::utils::{get_centy_path, CENTY_VERSION};
use std::path::Path;
/// Enrich a tracked project with live data from disk
pub async fn enrich_project(path: &str, tracked: &TrackedProject, org_name: Option<String>) -> ProjectInfo {
    let project_path = Path::new(path);
    let centy_path = get_centy_path(project_path);
    let manifest_path = centy_path.join(".centy-manifest.json");
    let initialized = manifest_path.exists();
    let issues_path = centy_path.join("issues");
    let issue_count = count_issues(&issues_path).await.unwrap_or(0);
    let docs_path = centy_path.join("docs");
    let doc_count = count_md_files(&docs_path).await.unwrap_or(0);
    let name = project_path.file_name().map(|n| n.to_string_lossy().to_string());
    let project_title = get_project_title(project_path).await;
    let (project_version, project_behind) = if initialized {
        match read_manifest(project_path).await {
            Ok(Some(manifest)) => { let behind = is_version_behind(&manifest.centy_version, CENTY_VERSION); (Some(manifest.centy_version), behind) }
            _ => (None, false),
        }
    } else { (None, false) };
    ProjectInfo {
        path: path.to_string(), first_accessed: tracked.first_accessed.clone(), last_accessed: tracked.last_accessed.clone(),
        issue_count, doc_count, initialized, name, is_favorite: tracked.is_favorite, is_archived: tracked.is_archived,
        organization_slug: tracked.organization_slug.clone(), organization_name: org_name,
        user_title: tracked.user_title.clone(), project_title, project_version, project_behind,
    }
}
pub fn is_version_behind(project_ver: &str, daemon_ver: &str) -> bool {
    use semver::Version;
    match (Version::parse(project_ver), Version::parse(daemon_ver)) {
        (Ok(pv), Ok(dv)) => pv < dv, _ => false,
    }
}
