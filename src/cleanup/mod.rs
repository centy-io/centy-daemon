#![allow(unknown_lints, max_lines_per_file)]
//! Background cleanup task: hard-delete soft-deleted artifacts past retention period.

use crate::config::item_type_config::discover_item_types_map;
use crate::config::read_config;
use crate::item::generic::storage::{generic_delete, generic_list};
use crate::registry::{list_projects, ListProjectsOptions};
use crate::utils::get_centy_path;
use chrono::{DateTime, Duration, Utc};
use mdstore::{Filters, TypeConfig};
use std::path::Path;
use tracing::{debug, error, info, warn};

/// Default retention period when `retention_period` is `None` or `"0"`.
const DEFAULT_RETENTION_DAYS: i64 = 30;

/// Parse a duration string like `"30d"`, `"24h"`, `"7d"` into a [`Duration`].
///
/// Returns `None` when the string is `"0"`, empty, or invalid.
/// Returns `Some(Duration)` with positive value for valid strings.
pub fn parse_retention_duration(s: &str) -> Option<Duration> {
    let s = s.trim();
    if s.is_empty() || s == "0" {
        return None;
    }
    if let Some(days) = s.strip_suffix('d') {
        let n: i64 = days.trim().parse().ok()?;
        if n <= 0 {
            return None;
        }
        return Some(Duration::days(n));
    }
    if let Some(hours) = s.strip_suffix('h') {
        let n: i64 = hours.trim().parse().ok()?;
        if n <= 0 {
            return None;
        }
        return Some(Duration::hours(n));
    }
    None
}

/// Run hard-delete cleanup on a single project.
///
/// Iterates all item types with soft-delete enabled, finds items whose
/// `deleted_at` timestamp is older than `retention`, and permanently removes them.
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

/// Run cleanup across all registered projects.
///
/// For each project in the global registry, reads its config and runs
/// `run_cleanup_for_project` if auto-cleanup is enabled.
pub async fn run_cleanup_all_projects() {
    let projects = match list_projects(ListProjectsOptions {
        include_archived: true,
        include_stale: false,
        include_uninitialized: false,
        include_temp: false,
        ungrouped_only: false,
        organization_slug: None,
    })
    .await
    {
        Ok(p) => p,
        Err(e) => {
            warn!(error = %e, "Failed to list projects for cleanup");
            return;
        }
    };

    for project in &projects {
        let project_path_str = &project.path;
        let project_path = Path::new(project_path_str);
        if !project_path.exists() {
            continue;
        }

        let config = match read_config(project_path).await {
            Ok(Some(c)) => c,
            Ok(None) => crate::config::CentyConfig::default(),
            Err(e) => {
                warn!(project = %project_path_str, error = %e, "Failed to read project config for cleanup");
                continue;
            }
        };

        let retention = match &config.cleanup.retention_period {
            // Explicitly null → disabled
            Some(s) if s.trim() == "null" => {
                debug!(project = %project_path_str, "Cleanup disabled (retention_period = null)");
                continue;
            }
            // "0" or empty → disabled
            Some(s) if s.trim() == "0" || s.trim().is_empty() => {
                debug!(project = %project_path_str, "Cleanup disabled (retention_period = 0)");
                continue;
            }
            Some(s) => match parse_retention_duration(s) {
                Some(d) => d,
                None => {
                    warn!(project = %project_path_str, value = %s, "Invalid retention_period, skipping cleanup");
                    continue;
                }
            },
            // None → use default
            None => Duration::days(DEFAULT_RETENTION_DAYS),
        };

        info!(project = %project_path_str, retention_days = retention.num_days(), "Running artifact cleanup");
        run_cleanup_for_project(project_path, retention).await;
    }
}

/// Spawn a background task that runs cleanup at startup and then every hour.
pub fn spawn_cleanup_task() {
    tokio::spawn(async {
        run_cleanup_all_projects().await;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            run_cleanup_all_projects().await;
        }
    });
}

#[cfg(test)]
#[path = "cleanup_tests.rs"]
mod cleanup_tests;
