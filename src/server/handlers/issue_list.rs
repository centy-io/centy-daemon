use std::path::Path;

use crate::config::read_config;
use crate::registry::track_project_async;
use crate::server::convert_entity::issue_to_proto;
use crate::server::proto::{ListIssuesRequest, ListIssuesResponse};
use tonic::{Response, Status};

pub async fn list_issues(req: ListIssuesRequest) -> Result<Response<ListIssuesResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);

    // Read config for priority_levels (for label generation)
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map_or(3, |c| c.priority_levels);

    let status_filter = if req.status.is_empty() {
        None
    } else {
        Some(req.status.as_str())
    };
    // Convert int32 priority filter: 0 means no filter
    let priority_filter = if req.priority == 0 {
        None
    } else {
        Some(req.priority as u32)
    };
    // Draft filter is optional bool
    let draft_filter = req.draft;

    match crate::item::entities::issue::list_issues(
        project_path,
        status_filter,
        priority_filter,
        draft_filter,
        false,
    )
    .await
    {
        Ok(issues) => {
            let total_count = issues.len() as i32;
            Ok(Response::new(ListIssuesResponse {
                issues: issues
                    .into_iter()
                    .map(|i| issue_to_proto(&i, priority_levels))
                    .collect(),
                total_count,
                success: true,
                error: String::new(),
            }))
        }
        Err(e) => Ok(Response::new(ListIssuesResponse {
            success: false,
            error: e.to_string(),
            issues: vec![],
            total_count: 0,
        })),
    }
}
