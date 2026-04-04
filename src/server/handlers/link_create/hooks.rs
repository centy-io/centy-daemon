use crate::hooks::{HookError, HookOperation};
use crate::server::hooks_helper::maybe_run_pre_hooks;
use std::path::Path;

pub(super) async fn run_pre_hooks(
    project_path: &Path,
    project_path_str: &str,
    item_id: &str,
    request_data: serde_json::Value,
) -> Result<(), HookError> {
    maybe_run_pre_hooks(
        project_path,
        "link",
        HookOperation::Create,
        project_path_str,
        Some(item_id),
        Some(request_data),
    )
    .await
}
