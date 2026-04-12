use crate::item::entities::issue::{update_issue, UpdateIssueOptions};
use std::path::Path;

pub(super) async fn try_update_status_on_open(
    config: Option<&crate::config::CentyConfig>,
    project_path: &Path,
    issue_id: &str,
    current_status: &str,
) {
    if let Some(cfg) = config {
        if cfg.workspace.update_status_on_open == Some(true)
            && current_status != "in-progress"
            && current_status != "closed"
        {
            drop(
                update_issue(
                    project_path,
                    issue_id,
                    UpdateIssueOptions {
                        status: Some("in-progress".to_string()),
                        ..Default::default()
                    },
                )
                .await,
            );
        }
    }
}
