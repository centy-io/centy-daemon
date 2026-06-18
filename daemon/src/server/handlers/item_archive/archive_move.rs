use super::operation::{do_move_to_archive, err_resp, set_original_item_type_and_respond};
use crate::hooks::HookOperation;
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::ArchiveItemResponse;
use mdstore::TypeConfig;
use std::path::Path;
use tonic::{Response, Status};

pub(super) async fn run_archive_hooks_and_move(
    project_path: &Path,
    project_path_str: &str,
    source_type: &str,
    archived_type: &str,
    source_config: &TypeConfig,
    archived_config: &TypeConfig,
    hook_type: &str,
    item_id: &str,
    hook_request_data: serde_json::Value,
) -> Result<Response<ArchiveItemResponse>, Status> {
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        hook_type,
        HookOperation::Move,
        project_path_str,
        Some(item_id),
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(err_resp(project_path_str, &e)));
    }
    let move_result = do_move_to_archive(
        project_path,
        source_type,
        archived_type,
        source_config,
        archived_config,
        item_id,
    )
    .await;
    let success = move_result.is_ok();
    maybe_run_post_hooks(
        project_path,
        hook_type,
        HookOperation::Move,
        project_path_str,
        Some(item_id),
        Some(hook_request_data),
        success,
    )
    .await;
    match move_result {
        Ok(result) => {
            set_original_item_type_and_respond(
                project_path,
                project_path_str,
                archived_type,
                archived_config,
                source_type,
                result.item,
            )
            .await
        }
        Err(e) => Ok(Response::new(err_resp(project_path_str, &e))),
    }
}
