use super::super::config::{HookDefinition, HookOperation, Phase};
use super::super::context::HookContext;
use super::common::{find_matching_hooks, load_hooks_config};
use super::record::run_recorded;
use std::path::Path;
use tracing::{debug, warn};

fn spawn_async_hook(hook: &HookDefinition, context: &HookContext, project_path: &Path) {
    let owned = hook.clone();
    let ctx = context.clone();
    let path = project_path.to_path_buf();
    tokio::spawn(async move {
        let pattern = &owned.pattern;
        match run_recorded(&owned, &ctx, &path, false).await {
            Ok(result) if result.exit_code != 0i32 => {
                debug!(
                    "Async post-hook '{}' exited with code {}: {}",
                    pattern, result.exit_code, result.stderr
                );
            }
            Err(e) => debug!("Async post-hook '{}' failed: {}", pattern, e),
            _ => {}
        }
    });
}

async fn run_sync_hook(hook: &HookDefinition, context: &HookContext, project_path: &Path) {
    match run_recorded(hook, context, project_path, false).await {
        Ok(result) if result.exit_code != 0i32 => {
            warn!(
                "Post-hook '{}' exited with code {}: {}",
                hook.pattern, result.exit_code, result.stderr
            );
        }
        Err(e) => warn!("Post-hook '{}' failed: {}", hook.pattern, e),
        _ => {}
    }
}

/// Run post-hooks for the given `item_type` and operation.
/// Synchronous post-hooks run inline (failures logged as warnings).
/// Async post-hooks are spawned in background (failures logged as debug).
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
            spawn_async_hook(hook, context, project_path);
        } else {
            run_sync_hook(hook, context, project_path).await;
        }
    }
}
