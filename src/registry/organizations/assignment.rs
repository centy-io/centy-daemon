use super::errors::OrganizationError;
use super::org_file::write_project_org_file;
use crate::registry::storage::{get_lock, read_registry, write_registry_unlocked};
use crate::registry::tracking::enrich_project;
use crate::registry::types::{ProjectInfo, ProjectOrganization, TrackedProject};
use crate::registry::validation::ValidationService;
use crate::registry::RegistryError;
use crate::utils::{get_centy_path, now_iso};
use std::path::Path;
use tokio::fs;

/// Set or remove a project's organization assignment
pub async fn set_project_organization(
    project_path: &str,
    org_slug: Option<&str>,
) -> Result<ProjectInfo, OrganizationError> {
    let path = Path::new(project_path);
    let canonical_path = path.canonicalize().map_or_else(
        |_| project_path.to_string(),
        |p| p.to_string_lossy().to_string(),
    );

    let _guard = get_lock().lock().await;
    let mut registry = read_registry().await?;

    // Verify org exists if assigning
    let org_name = if let Some(slug) = org_slug {
        if slug.is_empty() {
            None
        } else {
            let org = registry
                .organizations
                .get(slug)
                .ok_or_else(|| OrganizationError::NotFound(slug.to_string()))?;
            Some(org.name.clone())
        }
    } else {
        None
    };

    // Check for duplicate project names within organization
    if let Some(slug) = org_slug {
        if !slug.is_empty() {
            if let Some(project_name) = path.file_name() {
                let project_name_str = project_name.to_string_lossy();
                ValidationService::validate_unique_project_name(
                    &registry,
                    slug,
                    &canonical_path,
                    &project_name_str,
                )?;
            }
        }
    }

    let project = registry
        .projects
        .entry(canonical_path.clone())
        .or_insert_with(|| {
            let now = now_iso();
            TrackedProject {
                first_accessed: now.clone(),
                last_accessed: now,
                is_favorite: false,
                is_archived: false,
                organization_slug: None,
                user_title: None,
            }
        });
    project.organization_slug = org_slug.filter(|s| !s.is_empty()).map(String::from);

    let centy_path = get_centy_path(path);
    let org_file_path = centy_path.join("organization.json");
    if let Some(slug) = org_slug {
        if !slug.is_empty() {
            if let Some(org) = registry.organizations.get(slug) {
                let project_org = ProjectOrganization {
                    slug: slug.to_string(),
                    name: org.name.clone(),
                    description: org.description.clone(),
                };
                write_project_org_file(&org_file_path, &project_org).await?;
            }
        }
    } else if org_file_path.exists() {
        let _ = fs::remove_file(&org_file_path).await;
    }

    let Some(project) = registry.projects.get(&canonical_path) else {
        return Err(OrganizationError::RegistryError(
            RegistryError::ProjectNotFound(canonical_path),
        ));
    };
    let info = enrich_project(&canonical_path, project, org_name).await;

    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;

    Ok(info)
}
