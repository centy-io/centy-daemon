use crate::hooks::HookOperation;
use crate::item::generic::storage::generic_delete;
use crate::server::hooks_helper::maybe_run_post_hooks;
use crate::server::proto::DeleteItemResponse;
use crate::server::structured_error::to_error_json;
use mdstore::TypeConfig;
use std::path::Path;
pub(super) async fn do_delete(
    project_path: &Path,
    item_type: &str,
    config: &TypeConfig,
    item_id: &str,
    force: bool,
    hook_type: &str,
    hook_project_path: &str,
    hook_item_id: &str,
    hook_data: serde_json::Value,
    project_path_str: &str,
) -> DeleteItemResponse {
    match generic_delete(project_path, item_type, config, item_id, force).await {
        Ok(()) => {
            maybe_run_post_hooks(
                project_path,
                hook_type,
                HookOperation::Delete,
                hook_project_path,
                Some(hook_item_id),
                Some(hook_data),
                true,
            )
            .await;
            DeleteItemResponse {
                success: true,
                error: String::new(),
            }
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                hook_type,
                HookOperation::Delete,
                hook_project_path,
                Some(hook_item_id),
                Some(hook_data),
                false,
            )
            .await;
            DeleteItemResponse {
                success: false,
                error: to_error_json(project_path_str, &e),
            }
        }
    }
}
