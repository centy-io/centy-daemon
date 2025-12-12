use super::storage::{get_lock, read_registry, write_registry_unlocked};
use super::types::{Organization, OrganizationInfo, ProjectOrganization, TrackedProject};
use super::RegistryError;
use crate::utils::{get_centy_path, now_iso};
use std::path::Path;
use thiserror::Error;
use tokio::fs;
use tracing::info;

#[derive(Error, Debug)]
pub enum OrganizationError {
    #[error("Organization already exists: {0}")]
    AlreadyExists(String),

    #[error("Organization not found: {0}")]
    NotFound(String),

    #[error("Organization has {0} projects. Reassign or remove them first.")]
    HasProjects(u32),

    #[error("Invalid slug: {0}")]
    InvalidSlug(String),

    #[error("Registry error: {0}")]
    RegistryError(#[from] RegistryError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Convert a name to a URL-friendly slug (kebab-case)
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Validate a slug (must be non-empty, lowercase alphanumeric with hyphens)
fn validate_slug(slug: &str) -> Result<(), OrganizationError> {
    if slug.is_empty() {
        return Err(OrganizationError::InvalidSlug(
            "Slug cannot be empty".to_string(),
        ));
    }

    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(OrganizationError::InvalidSlug(
            "Slug must contain only lowercase letters, numbers, and hyphens".to_string(),
        ));
    }

    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(OrganizationError::InvalidSlug(
            "Slug cannot start or end with a hyphen".to_string(),
        ));
    }

    Ok(())
}

/// Create a new organization
pub async fn create_organization(
    slug: Option<&str>,
    name: &str,
    description: Option<&str>,
) -> Result<OrganizationInfo, OrganizationError> {
    let slug = match slug {
        Some(s) if !s.is_empty() => {
            validate_slug(s)?;
            s.to_string()
        }
        _ => slugify(name),
    };

    validate_slug(&slug)?;

    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;

    if registry.organizations.contains_key(&slug) {
        return Err(OrganizationError::AlreadyExists(slug));
    }

    let now = now_iso();
    let org = Organization {
        name: name.to_string(),
        description: description.map(String::from),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    registry.organizations.insert(slug.clone(), org.clone());
    registry.updated_at = now;
    write_registry_unlocked(&registry).await?;

    info!("Created organization: {}", slug);

    Ok(OrganizationInfo {
        slug,
        name: org.name,
        description: org.description,
        created_at: org.created_at,
        updated_at: org.updated_at,
        project_count: 0,
    })
}

/// List all organizations with their project counts
pub async fn list_organizations() -> Result<Vec<OrganizationInfo>, OrganizationError> {
    let registry = read_registry().await?;

    let mut orgs: Vec<OrganizationInfo> = registry
        .organizations
        .iter()
        .map(|(slug, org)| {
            let project_count = registry
                .projects
                .values()
                .filter(|p| p.organization_slug.as_deref() == Some(slug.as_str()))
                .count() as u32;

            OrganizationInfo {
                slug: slug.clone(),
                name: org.name.clone(),
                description: org.description.clone(),
                created_at: org.created_at.clone(),
                updated_at: org.updated_at.clone(),
                project_count,
            }
        })
        .collect();

    // Sort by name
    orgs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(orgs)
}

/// Get a specific organization by slug
pub async fn get_organization(
    slug: &str,
) -> Result<Option<OrganizationInfo>, OrganizationError> {
    let registry = read_registry().await?;

    if let Some(org) = registry.organizations.get(slug) {
        let project_count = registry
            .projects
            .values()
            .filter(|p| p.organization_slug.as_deref() == Some(slug))
            .count() as u32;

        Ok(Some(OrganizationInfo {
            slug: slug.to_string(),
            name: org.name.clone(),
            description: org.description.clone(),
            created_at: org.created_at.clone(),
            updated_at: org.updated_at.clone(),
            project_count,
        }))
    } else {
        Ok(None)
    }
}

/// Update an existing organization
pub async fn update_organization(
    slug: &str,
    name: Option<&str>,
    description: Option<&str>,
    new_slug: Option<&str>,
) -> Result<OrganizationInfo, OrganizationError> {
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;

    let org = registry
        .organizations
        .get_mut(slug)
        .ok_or_else(|| OrganizationError::NotFound(slug.to_string()))?;

    let now = now_iso();

    if let Some(n) = name {
        org.name = n.to_string();
    }

    if let Some(d) = description {
        org.description = if d.is_empty() { None } else { Some(d.to_string()) };
    }

    org.updated_at = now.clone();

    // Handle slug rename
    let final_slug = if let Some(ns) = new_slug {
        if !ns.is_empty() && ns != slug {
            validate_slug(ns)?;

            if registry.organizations.contains_key(ns) {
                return Err(OrganizationError::AlreadyExists(ns.to_string()));
            }

            // Remove org from old slug
            let org = registry.organizations.remove(slug).unwrap();

            // Update all projects that reference this org
            for project in registry.projects.values_mut() {
                if project.organization_slug.as_deref() == Some(slug) {
                    project.organization_slug = Some(ns.to_string());
                }
            }

            // Insert at new slug
            registry.organizations.insert(ns.to_string(), org);

            ns.to_string()
        } else {
            slug.to_string()
        }
    } else {
        slug.to_string()
    };

    let org = registry.organizations.get(&final_slug).unwrap();
    let project_count = registry
        .projects
        .values()
        .filter(|p| p.organization_slug.as_deref() == Some(final_slug.as_str()))
        .count() as u32;

    let result = OrganizationInfo {
        slug: final_slug,
        name: org.name.clone(),
        description: org.description.clone(),
        created_at: org.created_at.clone(),
        updated_at: org.updated_at.clone(),
        project_count,
    };

    registry.updated_at = now;
    write_registry_unlocked(&registry).await?;

    info!("Updated organization: {}", result.slug);

    Ok(result)
}

/// Delete an organization (fails if it has projects assigned)
pub async fn delete_organization(slug: &str) -> Result<(), OrganizationError> {
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;

    if !registry.organizations.contains_key(slug) {
        return Err(OrganizationError::NotFound(slug.to_string()));
    }

    // Check for projects using this org
    let project_count = registry
        .projects
        .values()
        .filter(|p| p.organization_slug.as_deref() == Some(slug))
        .count() as u32;

    if project_count > 0 {
        return Err(OrganizationError::HasProjects(project_count));
    }

    registry.organizations.remove(slug);
    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;

    info!("Deleted organization: {}", slug);

    Ok(())
}

/// Set or remove a project's organization assignment
pub async fn set_project_organization(
    project_path: &str,
    org_slug: Option<&str>,
) -> Result<super::types::ProjectInfo, OrganizationError> {
    let path = Path::new(project_path);

    // Canonicalize path to ensure consistent keys
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

    // Get or create project entry
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
            }
        });

    project.organization_slug = org_slug.filter(|s| !s.is_empty()).map(String::from);

    // Write project org file if assigning, remove if unassigning
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
                write_project_org_file_internal(&org_file_path, &project_org).await?;
            }
        }
    } else {
        // Remove the org file if it exists
        if org_file_path.exists() {
            let _ = fs::remove_file(&org_file_path).await;
        }
    }

    let project = registry.projects.get(&canonical_path).unwrap();

    // Enrich project info
    let info = super::tracking::enrich_project(&canonical_path, project, org_name).await;

    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;

    Ok(info)
}

/// Read the .centy/organization.json file from a project
pub async fn read_project_org_file(
    project_path: &Path,
) -> Result<Option<ProjectOrganization>, OrganizationError> {
    let centy_path = get_centy_path(project_path);
    let org_file_path = centy_path.join("organization.json");

    if !org_file_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&org_file_path).await?;
    let org: ProjectOrganization = serde_json::from_str(&content)?;

    Ok(Some(org))
}

/// Write the .centy/organization.json file
async fn write_project_org_file_internal(
    path: &Path,
    org: &ProjectOrganization,
) -> Result<(), OrganizationError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let content = serde_json::to_string_pretty(org)?;
    fs::write(path, content).await?;

    Ok(())
}

/// Sync an organization from a project's .centy/organization.json file
/// Called when a project is first accessed (e.g., after cloning a repo)
/// If the org doesn't exist globally, it's auto-imported
pub async fn sync_org_from_project(
    project_path: &Path,
) -> Result<Option<Organization>, OrganizationError> {
    let project_org = match read_project_org_file(project_path).await? {
        Some(org) => org,
        None => return Ok(None),
    };

    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;

    // Check if org already exists
    if let Some(existing) = registry.organizations.get(&project_org.slug) {
        return Ok(Some(existing.clone()));
    }

    // Auto-import the organization
    info!(
        "Auto-importing organization '{}' from project {}",
        project_org.slug,
        project_path.display()
    );

    let now = now_iso();
    let org = Organization {
        name: project_org.name,
        description: project_org.description,
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    registry
        .organizations
        .insert(project_org.slug.clone(), org.clone());

    // Also link the project to this org
    let canonical_path = project_path.canonicalize().map_or_else(
        |_| project_path.to_string_lossy().to_string(),
        |p| p.to_string_lossy().to_string(),
    );

    if let Some(project) = registry.projects.get_mut(&canonical_path) {
        project.organization_slug = Some(project_org.slug);
    }

    registry.updated_at = now;
    write_registry_unlocked(&registry).await?;

    Ok(Some(org))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Centy.io"), "centy-io");
        assert_eq!(slugify("My-Org"), "my-org");
        assert_eq!(slugify("test  spaces"), "test-spaces");
        assert_eq!(slugify("  leading"), "leading");
        assert_eq!(slugify("trailing  "), "trailing");
        assert_eq!(slugify("UPPERCASE"), "uppercase");
        assert_eq!(slugify("numbers123"), "numbers123");
    }

    #[test]
    fn test_validate_slug() {
        assert!(validate_slug("valid-slug").is_ok());
        assert!(validate_slug("also-valid-123").is_ok());
        assert!(validate_slug("simple").is_ok());

        assert!(validate_slug("").is_err());
        assert!(validate_slug("-start-with-hyphen").is_err());
        assert!(validate_slug("end-with-hyphen-").is_err());
        assert!(validate_slug("UPPERCASE").is_err());
        assert!(validate_slug("has spaces").is_err());
        assert!(validate_slug("has_underscore").is_err());
    }
}
