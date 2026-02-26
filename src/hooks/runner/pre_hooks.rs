use super::super::config::{HookOperation, Phase};
use super::super::context::HookContext;
use super::super::error::HookError;
use super::super::executor::execute_hook;
use super::common::{find_matching_hooks, load_hooks_config};
use std::path::Path;
use tracing::debug;
/// Run pre-hooks for the given item_type and operation.
/// Pre-hooks run synchronously; the first non-zero exit code aborts with an error.
pub async fn run_pre_hooks(
    project_path: &Path,
    item_type: &str,
    operation: HookOperation,
    context: &HookContext,
) -> Result<(), HookError> {
    let hooks = load_hooks_config(project_path).await;
    let matching = find_matching_hooks(&hooks, Phase::Pre, item_type, operation);
    if matching.is_empty() {
        return Ok(());
    }
    debug!(
        "Running {} pre-hooks for {}:{}",
        matching.len(),
        item_type,
        operation.as_str()
    );
    for hook in matching {
        let result = execute_hook(
            &hook.command,
            context,
            project_path,
            hook.timeout,
            &hook.pattern,
        )
        .await?;
        if result.exit_code != 0 {
            return Err(HookError::PreHookFailed {
                pattern: hook.pattern.clone(),
                exit_code: result.exit_code,
                stderr: result.stderr,
            });
        }
    }
    Ok(())
}
