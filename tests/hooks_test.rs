#![allow(clippy::indexing_slicing)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use centy_daemon::config::{write_config, CentyConfig};
use centy_daemon::hooks::config::{HookDefinition, HookOperation, ParsedPattern, Phase};
use centy_daemon::hooks::context::HookContext;
use centy_daemon::hooks::executor::execute_hook;
use centy_daemon::hooks::runner::{find_matching_hooks, run_post_hooks, run_pre_hooks};
use common::{create_test_dir, init_centy_project};

#[tokio::test]
async fn test_pre_hook_blocks_on_exit_1() {
    let temp_dir = create_test_dir();
    init_centy_project(temp_dir.path()).await;

    // Write config with a pre-hook that exits 1
    let mut config = CentyConfig::default();
    config.hooks.push(HookDefinition {
        pattern: "pre:issue:create".to_string(),
        command: "exit 1".to_string(),
        is_async: false,
        timeout: 30,
        enabled: true,
    });
    write_config(temp_dir.path(), &config).await.unwrap();

    let context = HookContext::new(
        Phase::Pre,
        "issue",
        HookOperation::Create,
        &temp_dir.path().to_string_lossy(),
        None,
        None,
        None,
    );

    let result = run_pre_hooks(temp_dir.path(), "issue", HookOperation::Create, &context).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("pre:issue:create"),
        "Error should mention the pattern: {err_msg}"
    );
}

#[tokio::test]
async fn test_pre_hook_passes_on_exit_0() {
    let temp_dir = create_test_dir();
    init_centy_project(temp_dir.path()).await;

    let mut config = CentyConfig::default();
    config.hooks.push(HookDefinition {
        pattern: "pre:issue:create".to_string(),
        command: "exit 0".to_string(),
        is_async: false,
        timeout: 30,
        enabled: true,
    });
    write_config(temp_dir.path(), &config).await.unwrap();

    let context = HookContext::new(
        Phase::Pre,
        "issue",
        HookOperation::Create,
        &temp_dir.path().to_string_lossy(),
        None,
        None,
        None,
    );

    let result = run_pre_hooks(temp_dir.path(), "issue", HookOperation::Create, &context).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_hook_receives_env_vars() {
    let temp_dir = create_test_dir();
    init_centy_project(temp_dir.path()).await;

    let marker = temp_dir.path().join("env_marker.txt");
    let command = format!(
        "echo \"$CENTY_PHASE $CENTY_ITEM_TYPE $CENTY_OPERATION $CENTY_ITEM_ID\" > {}",
        marker.to_string_lossy()
    );

    let mut config = CentyConfig::default();
    config.hooks.push(HookDefinition {
        pattern: "pre:issue:update".to_string(),
        command,
        is_async: false,
        timeout: 30,
        enabled: true,
    });
    write_config(temp_dir.path(), &config).await.unwrap();

    let context = HookContext::new(
        Phase::Pre,
        "issue",
        HookOperation::Update,
        &temp_dir.path().to_string_lossy(),
        Some("issue-abc"),
        None,
        None,
    );

    let result = run_pre_hooks(temp_dir.path(), "issue", HookOperation::Update, &context).await;

    assert!(result.is_ok());

    let content = tokio::fs::read_to_string(&marker).await.unwrap();
    assert_eq!(content.trim(), "pre issue update issue-abc");
}

#[tokio::test]
async fn test_hook_receives_stdin_json() {
    let temp_dir = create_test_dir();
    init_centy_project(temp_dir.path()).await;

    let marker = temp_dir.path().join("stdin_marker.txt");
    let command = format!("cat > {}", marker.to_string_lossy());

    let mut config = CentyConfig::default();
    config.hooks.push(HookDefinition {
        pattern: "pre:issue:create".to_string(),
        command,
        is_async: false,
        timeout: 30,
        enabled: true,
    });
    write_config(temp_dir.path(), &config).await.unwrap();

    let request_data = serde_json::json!({"title": "Test Issue"});
    let context = HookContext::new(
        Phase::Pre,
        "issue",
        HookOperation::Create,
        &temp_dir.path().to_string_lossy(),
        None,
        Some(request_data),
        None,
    );

    let result = run_pre_hooks(temp_dir.path(), "issue", HookOperation::Create, &context).await;

    assert!(result.is_ok());

    let content = tokio::fs::read_to_string(&marker).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["phase"], "pre");
    assert_eq!(parsed["item_type"], "issue");
    assert_eq!(parsed["operation"], "create");
    assert_eq!(parsed["request_data"]["title"], "Test Issue");
}

#[tokio::test]
async fn test_hook_timeout() {
    let temp_dir = create_test_dir();
    init_centy_project(temp_dir.path()).await;

    let context = HookContext::new(
        Phase::Pre,
        "issue",
        HookOperation::Create,
        &temp_dir.path().to_string_lossy(),
        None,
        None,
        None,
    );

    let result = execute_hook(
        "sleep 60",
        &context,
        temp_dir.path(),
        1, // 1 second timeout
        "pre:issue:create",
    )
    .await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("timed out"),
        "Error should mention timeout: {err_msg}"
    );
}

#[tokio::test]
async fn test_wildcard_matching_across_item_types() {
    let hooks = vec![HookDefinition {
        pattern: "*:*:delete".to_string(),
        command: "echo deleted".to_string(),
        is_async: false,
        timeout: 30,
        enabled: true,
    }];

    // Should match any item type with delete operation
    assert_eq!(
        find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Delete).len(),
        1
    );
    assert_eq!(
        find_matching_hooks(&hooks, Phase::Post, "doc", HookOperation::Delete).len(),
        1
    );
    assert_eq!(
        find_matching_hooks(&hooks, Phase::Post, "user", HookOperation::Delete).len(),
        1
    );
    // Should NOT match create
    assert_eq!(
        find_matching_hooks(&hooks, Phase::Pre, "issue", HookOperation::Create).len(),
        0
    );
}

#[tokio::test]
async fn test_specificity_ordering_verified_by_file() {
    let temp_dir = create_test_dir();
    init_centy_project(temp_dir.path()).await;

    let marker = temp_dir.path().join("order_marker.txt");
    let marker_path = marker.to_string_lossy().to_string();

    let mut config = CentyConfig::default();
    // Add hooks in reverse specificity order
    config.hooks.push(HookDefinition {
        pattern: "*:*:*".to_string(),
        command: format!("echo catch-all >> {marker_path}"),
        is_async: false,
        timeout: 30,
        enabled: true,
    });
    config.hooks.push(HookDefinition {
        pattern: "pre:issue:create".to_string(),
        command: format!("echo specific >> {marker_path}"),
        is_async: false,
        timeout: 30,
        enabled: true,
    });
    config.hooks.push(HookDefinition {
        pattern: "pre:*:create".to_string(),
        command: format!("echo mid >> {marker_path}"),
        is_async: false,
        timeout: 30,
        enabled: true,
    });
    write_config(temp_dir.path(), &config).await.unwrap();

    let context = HookContext::new(
        Phase::Pre,
        "issue",
        HookOperation::Create,
        &temp_dir.path().to_string_lossy(),
        None,
        None,
        None,
    );

    let result = run_pre_hooks(temp_dir.path(), "issue", HookOperation::Create, &context).await;

    assert!(result.is_ok());

    let content = tokio::fs::read_to_string(&marker).await.unwrap();
    let lines: Vec<&str> = content.trim().lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "specific"); // Most specific first (specificity 3)
    assert_eq!(lines[1], "mid"); // Mid specificity (specificity 2)
    assert_eq!(lines[2], "catch-all"); // Least specific (specificity 0)
}

#[tokio::test]
async fn test_disabled_hooks_are_skipped() {
    let temp_dir = create_test_dir();
    init_centy_project(temp_dir.path()).await;

    let marker = temp_dir.path().join("disabled_marker.txt");

    let mut config = CentyConfig::default();
    config.hooks.push(HookDefinition {
        pattern: "pre:issue:create".to_string(),
        command: format!("echo ran > {}", marker.to_string_lossy()),
        is_async: false,
        timeout: 30,
        enabled: false, // Disabled
    });
    write_config(temp_dir.path(), &config).await.unwrap();

    let context = HookContext::new(
        Phase::Pre,
        "issue",
        HookOperation::Create,
        &temp_dir.path().to_string_lossy(),
        None,
        None,
        None,
    );

    let result = run_pre_hooks(temp_dir.path(), "issue", HookOperation::Create, &context).await;

    assert!(result.is_ok());
    assert!(!marker.exists(), "Disabled hook should not have run");
}

#[tokio::test]
async fn test_no_hooks_configured_is_noop() {
    let temp_dir = create_test_dir();
    init_centy_project(temp_dir.path()).await;

    // Write config with no hooks
    let config = CentyConfig::default();
    write_config(temp_dir.path(), &config).await.unwrap();

    let context = HookContext::new(
        Phase::Pre,
        "issue",
        HookOperation::Create,
        &temp_dir.path().to_string_lossy(),
        None,
        None,
        None,
    );

    let result = run_pre_hooks(temp_dir.path(), "issue", HookOperation::Create, &context).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_post_hooks_run_after_success() {
    let temp_dir = create_test_dir();
    init_centy_project(temp_dir.path()).await;

    let marker = temp_dir.path().join("post_marker.txt");

    let mut config = CentyConfig::default();
    config.hooks.push(HookDefinition {
        pattern: "post:issue:create".to_string(),
        command: format!("echo post_ran > {}", marker.to_string_lossy()),
        is_async: false,
        timeout: 30,
        enabled: true,
    });
    write_config(temp_dir.path(), &config).await.unwrap();

    let context = HookContext::new(
        Phase::Post,
        "issue",
        HookOperation::Create,
        &temp_dir.path().to_string_lossy(),
        Some("issue-123"),
        None,
        Some(true),
    );

    run_post_hooks(temp_dir.path(), "issue", HookOperation::Create, &context).await;

    let content = tokio::fs::read_to_string(&marker).await.unwrap();
    assert_eq!(content.trim(), "post_ran");
}

#[test]
fn test_pattern_parse_all_operations() {
    // Verify all operations parse correctly
    let ops = [
        "create",
        "update",
        "delete",
        "soft-delete",
        "restore",
        "move",
        "duplicate",
    ];
    for op in ops {
        let pattern = format!("pre:issue:{op}");
        assert!(
            ParsedPattern::parse(&pattern).is_ok(),
            "Failed to parse pattern: {pattern}"
        );
    }
}

#[test]
fn test_pattern_parse_all_item_types() {
    // Built-in types
    let types = ["issue", "doc", "user", "link", "asset"];
    for item_type in types {
        let pattern = format!("pre:{item_type}:create");
        assert!(
            ParsedPattern::parse(&pattern).is_ok(),
            "Failed to parse pattern: {pattern}"
        );
    }

    // Custom types should also work
    let custom_types = ["epic", "pr", "widget", "ticket"];
    for item_type in custom_types {
        let pattern = format!("pre:{item_type}:create");
        assert!(
            ParsedPattern::parse(&pattern).is_ok(),
            "Custom type should parse: {pattern}"
        );
    }
}

#[test]
fn test_hook_definition_serialization() {
    let hook = HookDefinition {
        pattern: "pre:issue:create".to_string(),
        command: "echo test".to_string(),
        is_async: false,
        timeout: 30,
        enabled: true,
    };

    let json = serde_json::to_string(&hook).unwrap();
    let deserialized: HookDefinition = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.pattern, "pre:issue:create");
    assert_eq!(deserialized.command, "echo test");
    assert!(!deserialized.is_async);
    assert_eq!(deserialized.timeout, 30);
    assert!(deserialized.enabled);
}

#[test]
fn test_hook_definition_deserialization_defaults() {
    // Test that defaults work when fields are omitted
    let json = r#"{"pattern": "pre:issue:create", "command": "echo test"}"#;
    let hook: HookDefinition = serde_json::from_str(json).unwrap();

    assert_eq!(hook.pattern, "pre:issue:create");
    assert_eq!(hook.command, "echo test");
    assert!(!hook.is_async);
    assert_eq!(hook.timeout, 30); // default
    assert!(hook.enabled); // default
}

#[tokio::test]
async fn test_config_with_hooks_roundtrip() {
    let temp_dir = create_test_dir();
    init_centy_project(temp_dir.path()).await;

    let mut config = CentyConfig::default();
    config.hooks.push(HookDefinition {
        pattern: "pre:issue:create".to_string(),
        command: "echo before".to_string(),
        is_async: false,
        timeout: 10,
        enabled: true,
    });
    config.hooks.push(HookDefinition {
        pattern: "post:*:*".to_string(),
        command: "echo after".to_string(),
        is_async: true,
        timeout: 60,
        enabled: false,
    });

    write_config(temp_dir.path(), &config).await.unwrap();

    let loaded = centy_daemon::config::read_config(temp_dir.path())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(loaded.hooks.len(), 2);
    assert_eq!(loaded.hooks[0].pattern, "pre:issue:create");
    assert_eq!(loaded.hooks[0].command, "echo before");
    assert!(!loaded.hooks[0].is_async);
    assert_eq!(loaded.hooks[0].timeout, 10);
    assert!(loaded.hooks[0].enabled);

    assert_eq!(loaded.hooks[1].pattern, "post:*:*");
    assert!(loaded.hooks[1].is_async);
    assert!(!loaded.hooks[1].enabled);
}
