use std::path::Path;

use crate::config::read_config;
// Domain function accessed via fully-qualified path to avoid name conflict with handler
use crate::registry::track_project_async;
use crate::server::convert_entity::pr_to_proto;
use crate::server::proto::{ListPrsRequest, ListPrsResponse};
use tonic::{Response, Status};

pub async fn list_prs(req: ListPrsRequest) -> Result<Response<ListPrsResponse>, Status> {
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
    let source_filter = if req.source_branch.is_empty() {
        None
    } else {
        Some(req.source_branch.as_str())
    };
    let target_filter = if req.target_branch.is_empty() {
        None
    } else {
        Some(req.target_branch.as_str())
    };
    let priority_filter = if req.priority == 0 {
        None
    } else {
        Some(req.priority as u32)
    };

    match crate::item::entities::pr::list_prs(
        project_path,
        status_filter,
        source_filter,
        target_filter,
        priority_filter,
        false,
    )
    .await
    {
        Ok(prs) => {
            let total_count = prs.len() as i32;
            Ok(Response::new(ListPrsResponse {
                prs: prs
                    .into_iter()
                    .map(|p| pr_to_proto(&p, priority_levels))
                    .collect(),
                total_count,
            }))
        }
        Err(e) => Err(Status::internal(e.to_string())),
    }
}
