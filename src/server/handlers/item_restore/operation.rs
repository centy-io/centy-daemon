use crate::hooks::HookOperation;
use crate::item::generic::storage::{generic_get, generic_restore};
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::hooks_helper::maybe_run_post_hooks;
use crate::server::proto::RestoreItemResponse;
use crate::server::structured_error::to_error_json;
use std::path::Path;
pub(super) async fn do_restore(
    project_path: &Path,
    item_type: &str,
    item_id: &str,
    hook_type: &str,
    hook_project_path: &str,
    hook_item_id: &str,
    hook_data: serde_json::Value,
    project_path_str: &str,
) -> RestoreItemResponse {
    match generic_restore(project_path, item_type, item_id).await {
        Ok(()) => {
            maybe_run_post_hooks(
                project_path,
                hook_type,
                HookOperation::Restore,
                hook_project_path,
                Some(hook_item_id),
                Some(hook_data),
                true,
            )
            .await;
            let item = generic_get(project_path, item_type, item_id)
                .await
                .ok()
                .map(|i| generic_item_to_proto(&i, item_type));
            RestoreItemResponse {
                success: true,
                error: String::new(),
                item,
            }
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                hook_type,
                HookOperation::Restore,
                hook_project_path,
                Some(hook_item_id),
                Some(hook_data),
                false,
            )
            .await;
            RestoreItemResponse {
                success: false,
                error: to_error_json(project_path_str, &e),
                item: None,
            }
        }
    }
}
