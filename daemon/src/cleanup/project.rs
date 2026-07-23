use crate::config::item_type_config::{discover_item_types_map, ItemTypeRegistry};
use crate::item::generic::storage::{generic_delete, generic_list};
use crate::link::{delete_link_by_id, list_all_links, TargetType};
use crate::utils::get_centy_path;
use chrono::{DateTime, Duration, Utc};
use mdstore::{Filters, TypeConfig};
use std::path::Path;
use tracing::{debug, error, warn};

/// Resolve an entity type to its actual storage folder using the item type
/// registry (handles exact folder, case-insensitive name, and case-insensitive
/// folder matches). Falls back to the naive `folder_name()` (append "s") if
/// the registry has no matching type — matching the resolution used at link
/// creation time (see `link::crud_helpers::resolve_folder`).
fn resolve_folder(registry: Option<&ItemTypeRegistry>, entity_type: &TargetType) -> String {
    registry
        .and_then(|r| r.resolve(entity_type.as_str()).map(|(f, _)| f.clone()))
        .unwrap_or_else(|| entity_type.folder_name())
}

/// Returns `true` when either the source or the target item file no longer exists on disk.
fn link_is_orphan(
    centy_path: &std::path::Path,
    registry: Option<&ItemTypeRegistry>,
    link: &crate::link::LinkRecord,
) -> bool {
    let source_path = centy_path
        .join(resolve_folder(registry, &link.source_type))
        .join(format!("{}.md", link.source_id));
    let target_path = centy_path
        .join(resolve_folder(registry, &link.target_type))
        .join(format!("{}.md", link.target_id));
    !source_path.exists() || !target_path.exists()
}

/// Hard-delete a single orphan link, logging any failure.
async fn remove_orphan_link(project_path: &Path, link: &crate::link::LinkRecord) {
    debug!(
        project = %project_path.display(),
        link_id = %link.id,
        source_id = %link.source_id,
        target_id = %link.target_id,
        "Hard-deleting orphan link"
    );
    if let Err(e) = delete_link_by_id(project_path, &link.id).await {
        error!(link_id = %link.id, error = %e, "Failed to hard-delete orphan link");
    }
}

/// Hard-delete any link records whose source or target item no longer exists.
///
/// Runs as part of the regular cleanup pass and also on startup so that any
/// pre-existing orphan links (e.g., created before cascade-delete was
/// introduced) are removed.
pub async fn clean_orphan_links_for_project(project_path: &Path) {
    let centy_path = get_centy_path(project_path);
    let links = match list_all_links(project_path).await {
        Ok(l) => l,
        Err(e) => {
            warn!(project = %project_path.display(), error = %e, "Failed to list links for orphan cleanup");
            return;
        }
    };
    // Build once and reuse for every link so all configured item-type
    // directories (not just the naive singular+"s" guess) are considered
    // when checking whether an endpoint still exists.
    let registry = ItemTypeRegistry::build(project_path).await.ok();
    for link in links {
        if link_is_orphan(&centy_path, registry.as_ref(), &link) {
            remove_orphan_link(project_path, &link).await;
        }
    }
}

/// Run hard-delete cleanup on a single project.
///
/// Iterates all item types with soft-delete enabled, finds items whose
/// `deleted_at` timestamp is older than `retention`, and permanently removes them.
/// Also removes any orphan link records left over from earlier versions.
#[allow(clippy::cognitive_complexity)]
pub async fn run_cleanup_for_project(project_path: &Path, retention: Duration) {
    let centy_path = get_centy_path(project_path);
    let item_types = match discover_item_types_map(&centy_path).await {
        Ok(m) => m,
        Err(e) => {
            warn!(project = %project_path.display(), error = %e, "Failed to discover item types for cleanup");
            return;
        }
    };

    let now: DateTime<Utc> = Utc::now();

    for (folder, itc) in &item_types {
        if !itc.features.soft_delete {
            continue;
        }
        let type_config = TypeConfig::from(itc);
        let items = match generic_list(project_path, folder, Filters::new().include_deleted()).await
        {
            Ok(items) => items,
            Err(e) => {
                warn!(project = %project_path.display(), folder = %folder, error = %e, "Failed to list items for cleanup");
                continue;
            }
        };

        for item in items {
            let deleted_at_str = match &item.frontmatter.deleted_at {
                Some(s) => s.clone(),
                None => continue,
            };
            let deleted_at = match DateTime::parse_from_rfc3339(&deleted_at_str) {
                Ok(dt) => dt.with_timezone(&Utc),
                Err(e) => {
                    warn!(id = %item.id, deleted_at = %deleted_at_str, error = %e, "Invalid deleted_at, skipping");
                    continue;
                }
            };
            let age = now.signed_duration_since(deleted_at);
            if age >= retention {
                debug!(
                    project = %project_path.display(),
                    folder = %folder,
                    id = %item.id,
                    age_days = age.num_days(),
                    "Hard-deleting expired artifact"
                );
                if let Err(e) =
                    generic_delete(project_path, folder, &type_config, &item.id, true).await
                {
                    error!(id = %item.id, error = %e, "Failed to hard-delete expired artifact");
                }
            }
        }
    }

    // After expiring soft-deleted items, sweep for any orphan link records
    // that might have been left behind (e.g., from before cascade-delete).
    clean_orphan_links_for_project(project_path).await;
}
