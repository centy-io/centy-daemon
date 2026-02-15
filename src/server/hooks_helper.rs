use std::path::Path;

use crate::hooks::{run_post_hooks, run_pre_hooks, HookContext, HookError, HookOperation, Phase};

/// Build pre-hook context and run pre-hooks. Returns `Err(HookError)` if blocked.
pub async fn maybe_run_pre_hooks(
    project_path: &Path,
    item_type: &str,
    operation: HookOperation,
    project_path_str: &str,
    item_id: Option<&str>,
    request_data: Option<serde_json::Value>,
) -> Result<(), HookError> {
    let context = HookContext::new(
        Phase::Pre,
        item_type,
        operation,
        project_path_str,
        item_id,
        request_data,
        None,
    );
    run_pre_hooks(project_path, item_type, operation, &context).await
}

/// Build post-hook context and run post-hooks (sync ones block, async ones are spawned).
pub async fn maybe_run_post_hooks(
    project_path: &Path,
    item_type: &str,
    operation: HookOperation,
    project_path_str: &str,
    item_id: Option<&str>,
    request_data: Option<serde_json::Value>,
    success: bool,
) {
    let context = HookContext::new(
        Phase::Post,
        item_type,
        operation,
        project_path_str,
        item_id,
        request_data,
        Some(success),
    );
    run_post_hooks(project_path, item_type, operation, &context).await;
}
