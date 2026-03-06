use crate::config::item_type_config::discover_item_types_map;
use crate::item::generic::storage::{generic_delete, generic_list};
use crate::utils::get_centy_path;
use chrono::{DateTime, Duration, Utc};
use mdstore::{Filters, TypeConfig};
use std::path::Path;
use tracing::{debug, error, warn};

/// Run hard-delete cleanup on a single project.
///
/// Iterates all item types with soft-delete enabled, finds items whose
/// `deleted_at` timestamp is older than `retention`, and permanently removes them.
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
}
