use super::errors::OrganizationError;
use super::slug::{slugify, validate_slug};
use crate::registry::storage::{get_lock, read_registry, write_registry_unlocked};
use crate::registry::types::{Organization, OrganizationInfo};
use crate::utils::now_iso;
use tracing::info;

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
