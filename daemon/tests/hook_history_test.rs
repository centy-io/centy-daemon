#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod common;

use centy_daemon::hooks::config::{HookDefinition, HookOperation, HooksFile, Phase};
use centy_daemon::hooks::context::HookContext;
use centy_daemon::hooks::runner::{run_post_hooks, run_pre_hooks};
use centy_daemon::server::handlers::hook_execution_read::{
    get_hook_execution_handler, list_hook_executions_handler,
};
use centy_daemon::server::proto::{GetHookExecutionRequest, ListHookExecutionsRequest};
use common::{create_test_dir, init_centy_project};

fn hook(pattern: &str, command: &str, timeout: u64) -> HookDefinition {
    HookDefinition {
        pattern: pattern.to_string(),
        command: command.to_string(),
        is_async: false,
        timeout,
        enabled: true,
    }
}

fn ctx(phase: Phase, op: HookOperation, project: &std::path::Path) -> HookContext {
    HookContext::new(
        phase,
        "issue",
        op,
        &project.to_string_lossy(),
        Some("i-1"),
        None,
        None,
    )
}

fn list_req(project: &std::path::Path) -> ListHookExecutionsRequest {
    ListHookExecutionsRequest {
        project_path: project.to_string_lossy().to_string(),
        ..Default::default()
    }
}

#[tokio::test]
async fn test_hook_runs_are_recorded_and_readable_over_rpc() {
    let dir = create_test_dir();
    let p = dir.path();
    init_centy_project(p).await;
    let hooks = HooksFile {
        hooks: vec![
            hook("issue.creating", "echo out; echo err >&2; exit 3", 30),
            hook("issue.created", "echo done", 30),
            hook("issue.updating", "sleep 30", 1),
        ],
    };
    let yaml = serde_yaml::to_string(&hooks).unwrap();
    tokio::fs::write(p.join(".centy").join("hooks.yaml"), yaml)
        .await
        .unwrap();

    // Blocking pre-hook, then a post-hook, then a pre-hook that times out.
    run_pre_hooks(
        p,
        "issue",
        HookOperation::Create,
        &ctx(Phase::Pre, HookOperation::Create, p),
    )
    .await
    .unwrap_err();
    run_post_hooks(
        p,
        "issue",
        HookOperation::Create,
        &ctx(Phase::Post, HookOperation::Create, p),
    )
    .await;
    run_pre_hooks(
        p,
        "issue",
        HookOperation::Update,
        &ctx(Phase::Pre, HookOperation::Update, p),
    )
    .await
    .unwrap_err();

    // Newest first: timeout, post, blocked pre.
    let all = list_hook_executions_handler(list_req(p))
        .await
        .unwrap()
        .into_inner();
    assert!(all.success, "{}", all.error);
    let pats: Vec<_> = all
        .executions
        .iter()
        .map(|e| e.hook_pattern.as_str())
        .collect();
    assert_eq!(pats, ["issue.updating", "issue.created", "issue.creating"]);

    let timeout = &all.executions[0];
    assert!(timeout.timed_out && timeout.blocked_operation);
    assert_eq!(timeout.exit_code, -1i32);
    assert_eq!(timeout.operation, "update");

    let post = &all.executions[1];
    assert_eq!(post.stdout, "done\n");
    assert_eq!(post.exit_code, 0i32);
    assert!(!post.blocked_operation && !post.timed_out);
    assert_eq!(post.phase, "post");
    assert_eq!(post.item_id, "i-1");

    let pre = &all.executions[2];
    assert_eq!(pre.exit_code, 3i32);
    assert_eq!(pre.stdout, "out\n");
    assert_eq!(pre.stderr, "err\n");
    assert!(pre.blocked_operation && !pre.timed_out);
    assert_eq!(pre.command, "echo out; echo err >&2; exit 3");

    // Filters and limit.
    let mut req = list_req(p);
    req.phase = "post".to_string();
    let r = list_hook_executions_handler(req)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(r.executions.len(), 1);
    let mut req = list_req(p);
    req.operation = "create".to_string();
    req.limit = 1;
    let r = list_hook_executions_handler(req)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(r.executions.len(), 1);
    assert_eq!(r.executions[0].hook_pattern, "issue.created");

    // Get by id: found and not found.
    let found = get_hook_execution_handler(GetHookExecutionRequest {
        project_path: p.to_string_lossy().to_string(),
        execution_id: pre.id.clone(),
    })
    .await
    .unwrap()
    .into_inner();
    assert!(found.success && found.found);
    assert_eq!(found.execution.unwrap().hook_pattern, "issue.creating");
    let missing = get_hook_execution_handler(GetHookExecutionRequest {
        project_path: p.to_string_lossy().to_string(),
        execution_id: "nope".to_string(),
    })
    .await
    .unwrap()
    .into_inner();
    assert!(missing.success && !missing.found && missing.execution.is_none());
}

#[tokio::test]
async fn test_hook_history_empty_and_uninitialized() {
    let dir = create_test_dir();
    // Not initialized: structured error.
    let r = list_hook_executions_handler(list_req(dir.path()))
        .await
        .unwrap()
        .into_inner();
    assert!(!r.success);
    assert!(r.error.contains("NOT_INITIALIZED"), "{}", r.error);

    // Initialized but no hooks ever ran: empty success.
    init_centy_project(dir.path()).await;
    let r = list_hook_executions_handler(list_req(dir.path()))
        .await
        .unwrap()
        .into_inner();
    assert!(r.success && r.executions.is_empty());
}
