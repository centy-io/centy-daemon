use super::inference::try_auto_assign_organization;
use super::organizations::sync_org_from_project;
use super::storage::{get_lock, read_registry, write_registry_unlocked};
use super::types::{ListProjectsOptions, ProjectInfo, TrackedProject};
use super::RegistryError;
use crate::config::get_project_title;
use crate::utils::{get_centy_path, is_in_temp_dir, now_iso};
use std::path::Path;
use tokio::fs;
use tracing::warn;

/// Track a project access - called on any RPC operation
/// Updates `last_accessed` timestamp, creates new entry if not exists
pub async fn track_project(project_path: &str) -> Result<(), RegistryError> {
    let path = Path::new(project_path);

    // Canonicalize path to ensure consistent keys
    let canonical_path = path
        .canonicalize()
        .map_or_else(|_| project_path.to_string(), |p| p.to_string_lossy().to_string());

    // Track whether this project needs org inference (ungrouped)
    let needs_org_inference: bool;

    {
        // Lock the entire read-modify-write cycle to prevent race conditions
        let _guard = get_lock().lock().await;

        let mut registry = read_registry().await?;
        let now = now_iso();

        if let Some(entry) = registry.projects.get_mut(&canonical_path) {
            // Update existing entry (preserve is_favorite and organization_slug)
            entry.last_accessed = now.clone();
            needs_org_inference = entry.organization_slug.is_none();
        } else {
            // Create new entry
            let entry = TrackedProject {
                first_accessed: now.clone(),
                last_accessed: now.clone(),
                is_favorite: false,
                is_archived: false,
                organization_slug: None,
                user_title: None,
            };
            registry.projects.insert(canonical_path.clone(), entry);
            needs_org_inference = true;
        }

        registry.updated_at = now;
        write_registry_unlocked(&registry).await?;
    } // Lock released here

    // Fire-and-forget org inference for ungrouped projects
    if needs_org_inference {
        let path_for_inference = canonical_path;
        tokio::spawn(async move {
            let _ = try_auto_assign_organization(&path_for_inference, None).await;
        });
    }

    Ok(())
}

/// Track project access asynchronously (fire-and-forget)
/// Failures are logged but don't block the main operation
pub fn track_project_async(project_path: String) {
    tokio::spawn(async move {
        if let Err(e) = track_project(&project_path).await {
            warn!("Failed to track project {}: {}", project_path, e);
        }
    });
}

/// Remove a project from tracking
pub async fn untrack_project(project_path: &str) -> Result<(), RegistryError> {
    let path = Path::new(project_path);

    // Try canonical path first, fall back to original
    let canonical_path = path
        .canonicalize()
        .map_or_else(|_| project_path.to_string(), |p| p.to_string_lossy().to_string());

    // Lock the entire read-modify-write cycle to prevent race conditions
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;

    // Try to remove by canonical path first
    if registry.projects.remove(&canonical_path).is_none() {
        // If not found, try original path
        if registry.projects.remove(project_path).is_none() {
            return Err(RegistryError::ProjectNotFound(project_path.to_string()));
        }
    }

    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;

    Ok(())
}

/// Enrich a tracked project with live data from disk
pub async fn enrich_project(
    path: &str,
    tracked: &TrackedProject,
    org_name: Option<String>,
) -> ProjectInfo {
    let project_path = Path::new(path);
    let centy_path = get_centy_path(project_path);

    // Check if initialized (manifest exists)
    let manifest_path = centy_path.join(".centy-manifest.json");
    let initialized = manifest_path.exists();

    // Count issues (directories in .centy/issues/)
    let issues_path = centy_path.join("issues");
    let issue_count = count_directories(&issues_path).await.unwrap_or(0);

    // Count docs (markdown files in .centy/docs/)
    let docs_path = centy_path.join("docs");
    let doc_count = count_md_files(&docs_path).await.unwrap_or(0);

    // Get project name (directory name)
    let name = project_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string());

    // Get project-scope title from .centy/project.json
    let project_title = get_project_title(project_path).await;

    ProjectInfo {
        path: path.to_string(),
        first_accessed: tracked.first_accessed.clone(),
        last_accessed: tracked.last_accessed.clone(),
        issue_count,
        doc_count,
        initialized,
        name,
        is_favorite: tracked.is_favorite,
        is_archived: tracked.is_archived,
        organization_slug: tracked.organization_slug.clone(),
        organization_name: org_name,
        user_title: tracked.user_title.clone(),
        project_title,
    }
}

/// List all tracked projects, enriched with live data
pub async fn list_projects(opts: ListProjectsOptions<'_>) -> Result<Vec<ProjectInfo>, RegistryError> {
    let registry = read_registry().await?;

    let mut projects = Vec::new();
    // Collect ungrouped projects that exist on disk for background inference
    let mut ungrouped_paths: Vec<String> = Vec::new();

    for (path, tracked) in &registry.projects {
        let project_path = Path::new(path);
        let path_exists = project_path.exists();

        if !opts.include_stale && !path_exists {
            // Skip stale (non-existent) projects
            continue;
        }

        if !opts.include_temp && is_in_temp_dir(project_path) {
            // Skip projects in system temp directory
            continue;
        }

        if !opts.include_archived && tracked.is_archived {
            // Skip archived projects
            continue;
        }

        // Track ungrouped projects that exist for background inference
        if tracked.organization_slug.is_none() && path_exists {
            ungrouped_paths.push(path.clone());
        }

        // Filter by organization if specified
        if let Some(org_slug) = opts.organization_slug {
            if tracked.organization_slug.as_deref() != Some(org_slug) {
                continue;
            }
        }

        // Filter to ungrouped only if specified
        if opts.ungrouped_only && tracked.organization_slug.is_some() {
            continue;
        }

        // Get organization name if project has one
        let org_name = tracked
            .organization_slug
            .as_ref()
            .and_then(|slug| registry.organizations.get(slug))
            .map(|org| org.name.clone());

        // Try to sync org from project's organization.json (for clone workflow)
        if tracked.organization_slug.is_some() && org_name.is_none() {
            // Project has org slug but org doesn't exist - try to sync from file
            let project_path = Path::new(path);
            if let Ok(Some(_synced_slug)) = sync_org_from_project(project_path).await {
                // Re-read registry to get the org name after sync
                // For now, just use None - the next call will have the synced org
            }
        }

        let info = enrich_project(path, tracked, org_name).await;

        if !opts.include_uninitialized && !info.initialized {
            // Skip uninitialized projects
            continue;
        }

        projects.push(info);
    }

    // Sort by last_accessed (most recent first)
    projects.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));

    // Fire-and-forget org inference for ungrouped projects
    if !ungrouped_paths.is_empty() {
        tokio::spawn(async move {
            for path in ungrouped_paths {
                let _ = try_auto_assign_organization(&path, None).await;
            }
        });
    }

    Ok(projects)
}

/// Get info for a specific project
pub async fn get_project_info(project_path: &str) -> Result<Option<ProjectInfo>, RegistryError> {
    let path = Path::new(project_path);

    // Canonicalize path
    let canonical_path = path
        .canonicalize()
        .map_or_else(|_| project_path.to_string(), |p| p.to_string_lossy().to_string());

    let registry = read_registry().await?;

    // Try canonical path first, then original
    let tracked = registry
        .projects
        .get(&canonical_path)
        .or_else(|| registry.projects.get(project_path));

    match tracked {
        Some(tracked) => {
            // Get organization name if project has one
            let org_name = tracked
                .organization_slug
                .as_ref()
                .and_then(|slug| registry.organizations.get(slug))
                .map(|org| org.name.clone());

            Ok(Some(enrich_project(&canonical_path, tracked, org_name).await))
        }
        None => Ok(None),
    }
}

/// Count directories in a path (for counting issues)
async fn count_directories(path: &Path) -> Result<u32, std::io::Error> {
    if !path.exists() {
        return Ok(0);
    }

    let mut count = 0;
    let mut entries = fs::read_dir(path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            count += 1;
        }
    }

    Ok(count)
}

/// Count markdown files in a path (for counting docs)
async fn count_md_files(path: &Path) -> Result<u32, std::io::Error> {
    if !path.exists() {
        return Ok(0);
    }

    let mut count = 0;
    let mut entries = fs::read_dir(path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        if file_type.is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "md" {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

/// Set the favorite status for a project
pub async fn set_project_favorite(
    project_path: &str,
    is_favorite: bool,
) -> Result<ProjectInfo, RegistryError> {
    let path = Path::new(project_path);

    // Canonicalize path to ensure consistent keys
    let canonical_path = path
        .canonicalize()
        .map_or_else(|_| project_path.to_string(), |p| p.to_string_lossy().to_string());

    // Lock the entire read-modify-write cycle
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;

    // Determine which key to use (try canonical first, then original)
    let key = if registry.projects.contains_key(&canonical_path) {
        canonical_path.clone()
    } else if registry.projects.contains_key(project_path) {
        project_path.to_string()
    } else {
        return Err(RegistryError::ProjectNotFound(project_path.to_string()));
    };

    // Now we can safely get the mutable entry (key existence was checked above)
    let tracked = registry
        .projects
        .get_mut(&key)
        .expect("key was verified to exist");
    tracked.is_favorite = is_favorite;
    let tracked_clone = tracked.clone();

    // Get organization name
    let org_name = tracked_clone
        .organization_slug
        .as_ref()
        .and_then(|slug| registry.organizations.get(slug))
        .map(|org| org.name.clone());

    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;

    // Return enriched project info
    Ok(enrich_project(&canonical_path, &tracked_clone, org_name).await)
}

/// Set the archived status for a project
pub async fn set_project_archived(
    project_path: &str,
    is_archived: bool,
) -> Result<ProjectInfo, RegistryError> {
    let path = Path::new(project_path);

    // Canonicalize path to ensure consistent keys
    let canonical_path = path
        .canonicalize()
        .map_or_else(|_| project_path.to_string(), |p| p.to_string_lossy().to_string());

    // Lock the entire read-modify-write cycle
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;

    // Determine which key to use (try canonical first, then original)
    let key = if registry.projects.contains_key(&canonical_path) {
        canonical_path.clone()
    } else if registry.projects.contains_key(project_path) {
        project_path.to_string()
    } else {
        return Err(RegistryError::ProjectNotFound(project_path.to_string()));
    };

    // Now we can safely get the mutable entry (key existence was checked above)
    let tracked = registry
        .projects
        .get_mut(&key)
        .expect("key was verified to exist");
    tracked.is_archived = is_archived;
    let tracked_clone = tracked.clone();

    // Get organization name
    let org_name = tracked_clone
        .organization_slug
        .as_ref()
        .and_then(|slug| registry.organizations.get(slug))
        .map(|org| org.name.clone());

    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;

    // Return enriched project info
    Ok(enrich_project(&canonical_path, &tracked_clone, org_name).await)
}

/// Set the user-scope custom title for a project
/// This title is stored in the registry and is only visible to the current user
pub async fn set_project_user_title(
    project_path: &str,
    title: Option<String>,
) -> Result<ProjectInfo, RegistryError> {
    let path = Path::new(project_path);

    // Canonicalize path to ensure consistent keys
    let canonical_path = path
        .canonicalize()
        .map_or_else(|_| project_path.to_string(), |p| p.to_string_lossy().to_string());

    // Lock the entire read-modify-write cycle
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;

    // Determine which key to use (try canonical first, then original)
    let key = if registry.projects.contains_key(&canonical_path) {
        canonical_path.clone()
    } else if registry.projects.contains_key(project_path) {
        project_path.to_string()
    } else {
        return Err(RegistryError::ProjectNotFound(project_path.to_string()));
    };

    // Now we can safely get the mutable entry (key existence was checked above)
    let tracked = registry
        .projects
        .get_mut(&key)
        .expect("key was verified to exist");

    // Set title (None clears it, empty string also clears it)
    tracked.user_title = title.filter(|t| !t.is_empty());
    let tracked_clone = tracked.clone();

    // Get organization name
    let org_name = tracked_clone
        .organization_slug
        .as_ref()
        .and_then(|slug| registry.organizations.get(slug))
        .map(|org| org.name.clone());

    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;

    // Return enriched project info
    Ok(enrich_project(&canonical_path, &tracked_clone, org_name).await)
}

/// Get all projects belonging to an organization, optionally excluding a specific project
///
/// # Arguments
/// * `org_slug` - The organization slug to filter by
/// * `exclude_path` - Optional path to exclude from results (e.g., the calling project)
///
/// # Returns
/// List of initialized projects in the organization
pub async fn get_org_projects(
    org_slug: &str,
    exclude_path: Option<&str>,
) -> Result<Vec<ProjectInfo>, RegistryError> {
    let opts = ListProjectsOptions {
        include_stale: false,
        include_uninitialized: false,
        include_archived: false,
        organization_slug: Some(org_slug),
        ungrouped_only: false,
        include_temp: false,
    };

    let mut projects = list_projects(opts).await?;

    // Exclude the specified path if provided
    if let Some(exclude) = exclude_path {
        projects.retain(|p| p.path != exclude);
    }

    // Only return initialized projects
    projects.retain(|p| p.initialized);

    Ok(projects)
}
