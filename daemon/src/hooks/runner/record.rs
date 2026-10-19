use super::super::config::HookDefinition;
use super::super::context::HookContext;
use super::super::error::HookError;
use super::super::executor::{execute_hook, HookExecResult};
use super::super::history::{append_hook_execution, HookExecutionRecord};
use std::path::Path;

/// Run one hook and append its outcome to the project's execution history.
///
/// `can_block` marks pre-hooks: a non-zero exit or a timeout then sets `blocked_operation`.
pub async fn run_recorded(
    hook: &HookDefinition,
    context: &HookContext,
    project_path: &Path,
    can_block: bool,
) -> Result<HookExecResult, HookError> {
    let result = execute_hook(
        &hook.command,
        context,
        project_path,
        hook.timeout,
        &hook.pattern,
    )
    .await;
    let (exit_code, stdout, stderr, duration_ms, timed_out) = match &result {
        Ok(r) => (
            Some(r.exit_code),
            r.stdout.clone(),
            r.stderr.clone(),
            r.duration_ms,
            false,
        ),
        Err(HookError::Timeout { timeout_secs, .. }) => (
            None,
            String::new(),
            String::new(),
            timeout_secs.saturating_mul(1000),
            true,
        ),
        // Spawn/IO failures never ran the hook; there is nothing to record.
        Err(_) => return result,
    };
    let blocked_operation = can_block && (timed_out || exit_code != Some(0i32));
    let record = HookExecutionRecord {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        hook_pattern: hook.pattern.clone(),
        command: hook.command.clone(),
        exit_code,
        stdout,
        stderr,
        duration_ms,
        blocked_operation,
        phase: context.phase.clone(),
        item_type: context.item_type.clone(),
        operation: context.operation.clone(),
        item_id: context.item_id.clone(),
        timed_out,
    };
    append_hook_execution(project_path, &record).await;
    result
}
