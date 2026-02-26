use std::path::Path;
use crate::hooks::HookOperation;
use crate::item::generic::storage::generic_duplicate;
use crate::item::generic::types::DuplicateGenericItemOptions;
use crate::manifest::read_manifest;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::hooks_helper::maybe_run_post_hooks;
use crate::server::proto::DuplicateItemResponse;
use crate::server::structured_error::to_error_json;
use mdstore::TypeConfig;
pub(super) async fn do_duplicate(
    item_type: &str,
    config: &TypeConfig,
    hook_type: &str,
    hook_project_path: &str,
    hook_item_id: &str,
    hook_data: serde_json::Value,
    source_project_path_str: &str,
    target_project_path: &Path,
    options: DuplicateGenericItemOptions,
) -> DuplicateItemResponse {
    match generic_duplicate(item_type, config, options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                Path::new(hook_project_path), hook_type, HookOperation::Duplicate,
                hook_project_path, Some(hook_item_id), Some(hook_data), true,
            ).await;
            let manifest = read_manifest(target_project_path).await.ok().flatten();
            DuplicateItemResponse {
                success: true, error: String::new(),
                item: Some(generic_item_to_proto(&result.item, item_type)),
                original_id: result.original_id,
                manifest: manifest.as_ref().map(manifest_to_proto),
            }
        }
        Err(e) => {
            maybe_run_post_hooks(
                Path::new(hook_project_path), hook_type, HookOperation::Duplicate,
                hook_project_path, Some(hook_item_id), Some(hook_data), false,
            ).await;
            DuplicateItemResponse {
                success: false,
                error: to_error_json(source_project_path_str, &e),
                item: None, original_id: String::new(), manifest: None,
            }
        }
    }
}
