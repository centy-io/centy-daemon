//! Background cleanup task: hard-delete soft-deleted artifacts past retention period.

mod parse;
mod project;

pub use parse::parse_retention_duration;
pub use project::run_cleanup_for_project;

use crate::config::read_config;
use crate::registry::{list_projects, ListProjectsOptions};
use chrono::Duration;
use std::path::Path;
use tracing::{debug, info, warn};

/// Default retention period when `retention_period` is `None` or `"0"`.
const DEFAULT_RETENTION_DAYS: i64 = 30;

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
            Some(s) => {
                let Some(d) = parse_retention_duration(s) else {
                    warn!(project = %project_path_str, value = %s, "Invalid retention_period, skipping cleanup");
                    continue;
                };
                d
            }
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
