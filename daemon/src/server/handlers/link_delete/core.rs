use crate::hooks::HookOperation;
use crate::server::hooks_helper::maybe_run_post_hooks;
use crate::server::proto::DeleteLinkResponse;
use crate::server::structured_error::to_error_json;
use std::path::Path;
use tonic::{Response, Status};

pub async fn run_delete_link_by_id(
    project_path: &Path,
    link_id: &str,
    hook_project_path: String,
    hook_item_id: String,
    hook_request_data: serde_json::Value,
    cwd: &str,
) -> Result<Response<DeleteLinkResponse>, Status> {
    match crate::link::delete_link_by_id(project_path, link_id).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                "link",
                HookOperation::Delete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                true,
            )
            .await;
            Ok(Response::new(DeleteLinkResponse {
                success: true,
                error: String::new(),
                deleted_count: result.deleted_count,
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                "link",
                HookOperation::Delete,
                &hook_project_path,
                Some(&hook_item_id),
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(DeleteLinkResponse {
                success: false,
                error: to_error_json(cwd, &e),
                deleted_count: 0,
            }))
        }
    }
}
