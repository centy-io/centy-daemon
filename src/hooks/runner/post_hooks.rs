use super::super::config::{HookOperation, Phase};
use super::super::context::HookContext;
use super::super::executor::execute_hook;
use super::common::{find_matching_hooks, load_hooks_config};
use std::path::Path;
use tracing::{debug, warn};
/// Run post-hooks for the given item_type and operation.
/// Synchronous post-hooks run inline (failures logged as warnings).
/// Async post-hooks are spawned in background (failures logged as debug).
#[allow(
    unknown_lints,
    max_lines_per_function,
    clippy::too_many_lines,
    max_nesting_depth
)]
pub async fn run_post_hooks(
    project_path: &Path,
    item_type: &str,
    operation: HookOperation,
    context: &HookContext,
) {
    let hooks = load_hooks_config(project_path).await;
    let matching = find_matching_hooks(&hooks, Phase::Post, item_type, operation);
    if matching.is_empty() {
        return;
    }
    debug!(
        "Running {} post-hooks for {}:{}",
        matching.len(),
        item_type,
        operation.as_str()
    );
    for hook in matching {
        if hook.is_async {
            let command = hook.command.clone();
            let context = context.clone();
            let path = project_path.to_path_buf();
            let timeout = hook.timeout;
            let pattern = hook.pattern.clone();
            tokio::spawn(async move {
                match execute_hook(&command, &context, &path, timeout, &pattern).await {
                    Ok(result) if result.exit_code != 0 => {
                        debug!(
                            "Async post-hook '{}' exited with code {}: {}",
                            pattern, result.exit_code, result.stderr
                        );
                    }
                    Err(e) => {
                        debug!("Async post-hook '{}' failed: {}", pattern, e);
                    }
                    _ => {}
                }
            });
        } else {
            match execute_hook(
                &hook.command,
                context,
                project_path,
                hook.timeout,
                &hook.pattern,
            )
            .await
            {
                Ok(result) if result.exit_code != 0 => {
                    warn!(
                        "Post-hook '{}' exited with code {}: {}",
                        hook.pattern, result.exit_code, result.stderr
                    );
                }
                Err(e) => {
                    warn!("Post-hook '{}' failed: {}", hook.pattern, e);
                }
                _ => {}
            }
        }
    }
}
