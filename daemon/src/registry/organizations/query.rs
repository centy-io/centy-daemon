use super::errors::OrganizationError;
use crate::registry::storage::read_registry;
use crate::registry::types::OrganizationInfo;

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
                .count()
                .try_into()
                .unwrap_or(u32::MAX);

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
pub async fn get_organization(slug: &str) -> Result<Option<OrganizationInfo>, OrganizationError> {
    let registry = read_registry().await?;

    if let Some(org) = registry.organizations.get(slug) {
        let project_count = registry
            .projects
            .values()
            .filter(|p| p.organization_slug.as_deref() == Some(slug))
            .count()
            .try_into()
            .unwrap_or(u32::MAX);

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

#[cfg(test)]
#[path = "query_tests.rs"]
mod tests;
