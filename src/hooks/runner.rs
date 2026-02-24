use std::path::Path;
use tracing::{debug, warn};
use uuid::Uuid;

use super::config::{HookDefinition, HookOperation, ParsedPattern, Phase};
use super::context::HookContext;
use super::error::HookError;
use super::executor::execute_hook;
use super::history::{append_hook_execution, HookExecutionRecord};

/// Load hooks configuration from the project's config.json.
/// Returns an empty vec if no config exists or no hooks are configured.
pub async fn load_hooks_config(project_path: &Path) -> Vec<HookDefinition> {
    match crate::config::read_config(project_path).await {
        Ok(Some(config)) => config.hooks,
        _ => Vec::new(),
    }
}

/// Find matching hooks for the given phase, item_type, and operation.
/// Returns enabled hooks sorted by specificity descending (most-specific-first).
pub fn find_matching_hooks<'a>(
    hooks: &'a [HookDefinition],
    phase: Phase,
    item_type: &str,
    operation: HookOperation,
) -> Vec<&'a HookDefinition> {
    let mut matching: Vec<(&HookDefinition, u8)> = hooks
        .iter()
        .filter(|h| h.enabled)
        .filter_map(|h| {
            ParsedPattern::parse(&h.pattern)
                .ok()
                .filter(|p| p.matches(phase, item_type, operation))
                .map(|p| (h, p.specificity()))
        })
        .collect();

    // Sort by specificity descending (most-specific-first)
    matching.sort_by(|a, b| b.1.cmp(&a.1));
    matching.into_iter().map(|(h, _)| h).collect()
}

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
        let exec_result = execute_hook(
            &hook.command,
            context,
            project_path,
            hook.timeout,
            &hook.pattern,
        )
        .await;

        match exec_result {
            Ok(result) => {
                let blocked = result.exit_code != 0;
                let record = HookExecutionRecord {
                    id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    hook_pattern: hook.pattern.clone(),
                    command: hook.command.clone(),
                    exit_code: Some(result.exit_code),
                    stdout: result.stdout.clone(),
                    stderr: result.stderr.clone(),
                    duration_ms: result.duration_ms,
                    blocked_operation: blocked,
                    phase: context.phase.clone(),
                    item_type: context.item_type.clone(),
                    operation: context.operation.clone(),
                    item_id: context.item_id.clone(),
                    timed_out: false,
                };
                append_hook_execution(project_path, &record).await;

                if blocked {
                    return Err(HookError::PreHookFailed {
                        pattern: hook.pattern.clone(),
                        exit_code: result.exit_code,
                        stderr: result.stderr,
                    });
                }
            }
            Err(HookError::Timeout {
                ref pattern,
                timeout_secs,
            }) => {
                let record = HookExecutionRecord {
                    id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    hook_pattern: hook.pattern.clone(),
                    command: hook.command.clone(),
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                    duration_ms: timeout_secs * 1000,
                    blocked_operation: true,
                    phase: context.phase.clone(),
                    item_type: context.item_type.clone(),
                    operation: context.operation.clone(),
                    item_id: context.item_id.clone(),
                    timed_out: true,
                };
                append_hook_execution(project_path, &record).await;
                return Err(HookError::Timeout {
                    pattern: pattern.clone(),
                    timeout_secs,
                });
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

/// Run post-hooks for the given item_type and operation.
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
            // Spawn async hooks in background
            let command = hook.command.clone();
            let context = context.clone();
            let path = project_path.to_path_buf();
            let timeout = hook.timeout;
            let pattern = hook.pattern.clone();
            tokio::spawn(async move {
                match execute_hook(&command, &context, &path, timeout, &pattern).await {
                    Ok(result) => {
                        let record = HookExecutionRecord {
                            id: Uuid::new_v4().to_string(),
                            timestamp: chrono::Utc::now(),
                            hook_pattern: pattern.clone(),
                            command: command.clone(),
                            exit_code: Some(result.exit_code),
                            stdout: result.stdout.clone(),
                            stderr: result.stderr.clone(),
                            duration_ms: result.duration_ms,
                            blocked_operation: false,
                            phase: context.phase.clone(),
                            item_type: context.item_type.clone(),
                            operation: context.operation.clone(),
                            item_id: context.item_id.clone(),
                            timed_out: false,
                        };
                        append_hook_execution(&path, &record).await;
                        if result.exit_code != 0 {
                            debug!(
                                "Async post-hook '{}' exited with code {}: {}",
                                pattern, result.exit_code, result.stderr
                            );
                        }
                    }
                    Err(HookError::Timeout {
                        ref pattern,
                        timeout_secs,
                    }) => {
                        let record = HookExecutionRecord {
                            id: Uuid::new_v4().to_string(),
                            timestamp: chrono::Utc::now(),
                            hook_pattern: pattern.clone(),
                            command: command.clone(),
                            exit_code: None,
                            stdout: String::new(),
                            stderr: String::new(),
                            duration_ms: timeout_secs * 1000,
                            blocked_operation: false,
                            phase: context.phase.clone(),
                            item_type: context.item_type.clone(),
                            operation: context.operation.clone(),
                            item_id: context.item_id.clone(),
                            timed_out: true,
                        };
                        append_hook_execution(&path, &record).await;
                        debug!("Async post-hook '{}' timed out", pattern);
                    }
                    Err(e) => {
                        debug!("Async post-hook '{}' failed: {}", pattern, e);
                    }
                }
            });
        } else {
            // Run synchronous post-hooks inline
            match execute_hook(
                &hook.command,
                context,
                project_path,
                hook.timeout,
                &hook.pattern,
            )
            .await
            {
                Ok(result) => {
                    let record = HookExecutionRecord {
                        id: Uuid::new_v4().to_string(),
                        timestamp: chrono::Utc::now(),
                        hook_pattern: hook.pattern.clone(),
                        command: hook.command.clone(),
                        exit_code: Some(result.exit_code),
                        stdout: result.stdout.clone(),
                        stderr: result.stderr.clone(),
                        duration_ms: result.duration_ms,
                        blocked_operation: false,
                        phase: context.phase.clone(),
                        item_type: context.item_type.clone(),
                        operation: context.operation.clone(),
                        item_id: context.item_id.clone(),
                        timed_out: false,
                    };
                    append_hook_execution(project_path, &record).await;
                    if result.exit_code != 0 {
                        warn!(
                            "Post-hook '{}' exited with code {}: {}",
                            hook.pattern, result.exit_code, result.stderr
                        );
                    }
                }
                Err(HookError::Timeout {
                    ref pattern,
                    timeout_secs,
                }) => {
                    let record = HookExecutionRecord {
                        id: Uuid::new_v4().to_string(),
                        timestamp: chrono::Utc::now(),
                        hook_pattern: hook.pattern.clone(),
                        command: hook.command.clone(),
                        exit_code: None,
                        stdout: String::new(),
                        stderr: String::new(),
                        duration_ms: timeout_secs * 1000,
                        blocked_operation: false,
                        phase: context.phase.clone(),
                        item_type: context.item_type.clone(),
                        operation: context.operation.clone(),
                        item_id: context.item_id.clone(),
                        timed_out: true,
                    };
                    append_hook_execution(project_path, &record).await;
                    warn!("Post-hook '{}' timed out", pattern);
                }
                Err(e) => {
                    warn!("Post-hook '{}' failed: {}", hook.pattern, e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_hooks_empty() {
        let hooks: Vec<HookDefinition> = vec![];
        let result = find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Create);
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_matching_hooks_exact_match() {
        let hooks = vec![HookDefinition {
            pattern: "pre:issue:create".to_string(),
            command: "echo test".to_string(),
            is_async: false,
            timeout: 30,
            enabled: true,
        }];
        let result = find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Create);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_find_matching_hooks_no_match() {
        let hooks = vec![HookDefinition {
            pattern: "pre:issue:create".to_string(),
            command: "echo test".to_string(),
            is_async: false,
            timeout: 30,
            enabled: true,
        }];
        let result = find_matching_hooks(&hooks, Phase::Pre, "doc", HookOperation::Create);
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_matching_hooks_disabled_skipped() {
        let hooks = vec![HookDefinition {
            pattern: "pre:issue:create".to_string(),
            command: "echo test".to_string(),
            is_async: false,
            timeout: 30,
            enabled: false,
        }];
        let result = find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Create);
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_matching_hooks_specificity_order() {
        let hooks = vec![
            HookDefinition {
                pattern: "*:*:*".to_string(),
                command: "echo catch-all".to_string(),
                is_async: false,
                timeout: 30,
                enabled: true,
            },
            HookDefinition {
                pattern: "pre:issue:create".to_string(),
                command: "echo specific".to_string(),
                is_async: false,
                timeout: 30,
                enabled: true,
            },
            HookDefinition {
                pattern: "pre:*:create".to_string(),
                command: "echo mid".to_string(),
                is_async: false,
                timeout: 30,
                enabled: true,
            },
        ];
        let result = find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Create);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].command, "echo specific"); // specificity 3
        assert_eq!(result[1].command, "echo mid"); // specificity 2
        assert_eq!(result[2].command, "echo catch-all"); // specificity 0
    }

    #[test]
    fn test_find_matching_hooks_wildcard_matches_multiple() {
        let hooks = vec![HookDefinition {
            pattern: "*:*:delete".to_string(),
            command: "echo delete".to_string(),
            is_async: false,
            timeout: 30,
            enabled: true,
        }];
        // Should match for any item type with delete operation
        assert_eq!(
            find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Delete).len(),
            1
        );
        assert_eq!(
            find_matching_hooks(&hooks, Phase::Post, "doc", HookOperation::Delete).len(),
            1
        );
        assert_eq!(
            find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Create).len(),
            0
        );
    }
}
