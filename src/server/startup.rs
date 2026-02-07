use crate::registry::{list_projects, try_auto_assign_organization, ListProjectsOptions};
use tracing::info;

/// Background task to infer organizations for ungrouped projects on daemon startup.
pub async fn startup_org_inference() {
    use tokio::time::{sleep, Duration};

    // Small delay to let the daemon fully initialize
    sleep(Duration::from_millis(100)).await;

    // List all ungrouped projects that exist on disk
    let projects = match list_projects(ListProjectsOptions {
        include_stale: false,
        include_uninitialized: true,
        include_archived: false,
        ungrouped_only: true,
        ..Default::default()
    })
    .await
    {
        Ok(p) => p,
        Err(e) => {
            info!("Startup org inference: failed to list projects: {e}");
            return;
        }
    };

    if projects.is_empty() {
        return;
    }

    info!(
        "Startup org inference: scanning {} ungrouped projects",
        projects.len()
    );

    let mut inferred_count: u32 = 0;
    for project in projects {
        // Small delay between projects to avoid overloading
        sleep(Duration::from_millis(50)).await;

        if let Some(result) = try_auto_assign_organization(&project.path, None).await {
            if result.inferred_org_slug.is_some() && !result.has_mismatch {
                inferred_count = inferred_count.saturating_add(1);
            }
        }
    }

    if inferred_count > 0 {
        info!(
            "Startup org inference: assigned organizations to {} projects",
            inferred_count
        );
    }
}
