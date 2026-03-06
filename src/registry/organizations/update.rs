use super::errors::OrganizationError;
use super::slug::validate_slug;
use crate::registry::storage::{get_lock, read_registry, write_registry_unlocked};
use crate::registry::types::OrganizationInfo;
use crate::registry::ProjectRegistry;
use crate::utils::now_iso;
use tracing::info;

fn handle_slug_rename(
    registry: &mut ProjectRegistry,
    slug: &str,
    new_slug: Option<&str>,
) -> Result<String, OrganizationError> {
    let Some(ns) = new_slug else {
        return Ok(slug.to_string());
    };
    if !ns.is_empty() && ns != slug {
        validate_slug(ns)?;
        if registry.organizations.contains_key(ns) {
            return Err(OrganizationError::AlreadyExists(ns.to_string()));
        }
        let Some(org) = registry.organizations.remove(slug) else {
            return Err(OrganizationError::NotFound(slug.to_string()));
        };
        for project in registry.projects.values_mut() {
            if project.organization_slug.as_deref() == Some(slug) {
                project.organization_slug = Some(ns.to_string());
            }
        }
        registry.organizations.insert(ns.to_string(), org);
        Ok(ns.to_string())
    } else {
        Ok(slug.to_string())
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
        org.description = if d.is_empty() {
            None
        } else {
            Some(d.to_string())
        };
    }

    org.updated_at.clone_from(&now);

    let final_slug = handle_slug_rename(&mut registry, slug, new_slug)?;

    let Some(org) = registry.organizations.get(&final_slug) else {
        return Err(OrganizationError::NotFound(final_slug));
    };
    let project_count = registry
        .projects
        .values()
        .filter(|p| p.organization_slug.as_deref() == Some(final_slug.as_str()))
        .count()
        .try_into()
        .unwrap_or(u32::MAX);

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
