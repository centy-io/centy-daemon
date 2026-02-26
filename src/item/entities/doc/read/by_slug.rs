use std::path::Path;
use crate::registry::ProjectInfo;
use crate::item::entities::doc::error::DocError;
use crate::item::entities::doc::helpers::validate_slug;
use crate::item::entities::doc::types::{Doc, DocWithProject, GetDocsBySlugResult};
use super::get::get_doc;
/// Search for docs by slug across all tracked projects
pub async fn get_docs_by_slug(
    slug: &str,
    projects: &[ProjectInfo],
) -> Result<GetDocsBySlugResult, DocError> {
    validate_slug(slug)?;
    let mut found_docs = Vec::new();
    let mut errors = Vec::new();
    for project in projects {
        if !project.initialized { continue; }
        let project_path = Path::new(&project.path);
        match get_doc(project_path, slug).await {
            Ok(doc) => {
                let project_name = project.name.clone().unwrap_or_else(|| {
                    project_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| project.path.clone())
                });
                found_docs.push(DocWithProject {
                    doc,
                    project_path: project.path.clone(),
                    project_name,
                });
            }
            Err(DocError::DocNotFound(_)) => {}
            Err(DocError::NotInitialized) => {}
            Err(e) => {
                errors.push(format!("Error searching {}: {}", project.path, e));
            }
        }
    }
    Ok(GetDocsBySlugResult { docs: found_docs, errors })
}
