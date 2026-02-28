use super::super::storage::read_registry;
use super::super::types::ProjectInfo;
use super::super::RegistryError;
use super::enrich_fn::enrich_project;
use std::path::Path;
/// Get info for a specific project
pub async fn get_project_info(project_path: &str) -> Result<Option<ProjectInfo>, RegistryError> {
    let path = Path::new(project_path);
    let canonical_path = path.canonicalize().map_or_else(
        |_| project_path.to_string(),
        |p| p.to_string_lossy().to_string(),
    );
    let registry = read_registry().await?;
    let tracked = registry
        .projects
        .get(&canonical_path)
        .or_else(|| registry.projects.get(project_path));
    match tracked {
        Some(tracked) => {
            let org_name = tracked
                .organization_slug
                .as_ref()
                .and_then(|slug| registry.organizations.get(slug))
                .map(|org| org.name.clone());
            Ok(Some(
                enrich_project(&canonical_path, tracked, org_name).await,
            ))
        }
        None => Ok(None),
    }
}
