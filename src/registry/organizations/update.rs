use super::errors::OrganizationError;
use super::slug::validate_slug;
use crate::registry::storage::{get_lock, read_registry, write_registry_unlocked};
use crate::registry::types::OrganizationInfo;
use crate::utils::now_iso;
use tracing::info;

/// Update an existing organization
#[allow(unknown_lints, max_lines_per_function)]
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
        org.description = if d.is_empty() {
            None
        } else {
            Some(d.to_string())
        };
    }

    org.updated_at.clone_from(&now);

    // Handle slug rename
    let final_slug = if let Some(ns) = new_slug {
        if !ns.is_empty() && ns != slug {
            validate_slug(ns)?;

            if registry.organizations.contains_key(ns) {
                return Err(OrganizationError::AlreadyExists(ns.to_string()));
            }

            // Remove org from old slug (we verified it exists earlier)
            let Some(org) = registry.organizations.remove(slug) else {
                return Err(OrganizationError::NotFound(slug.to_string()));
            };

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

    let Some(org) = registry.organizations.get(&final_slug) else {
        return Err(OrganizationError::NotFound(final_slug));
    };
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
