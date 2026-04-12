use super::*;

#[test]
fn test_context_env_vars() {
    let ctx = HookContext::new(
        Phase::Pre,
        "issue",
        HookOperation::Create,
        "/tmp/project",
        Some("issue-123"),
        None,
        None,
    );

    let vars = ctx.to_env_vars();
    assert_eq!(&vars["CENTY_PHASE"], "pre");
    assert_eq!(&vars["CENTY_ITEM_TYPE"], "issue");
    assert_eq!(&vars["CENTY_OPERATION"], "create");
    assert_eq!(&vars["CENTY_PROJECT_PATH"], "/tmp/project");
    assert_eq!(&vars["CENTY_ITEM_ID"], "issue-123");
}

#[test]
fn test_context_env_vars_no_item_id() {
    let ctx = HookContext::new(
        Phase::Pre,
        "doc",
        HookOperation::Create,
        "/tmp/project",
        None,
        None,
        None,
    );

    let vars = ctx.to_env_vars();
    assert!(!vars.contains_key("CENTY_ITEM_ID"));
}

#[test]
fn test_context_to_json() {
    let ctx = HookContext::new(
        Phase::Post,
        "issue",
        HookOperation::Create,
        "/tmp/project",
        Some("issue-123"),
        Some(serde_json::json!({"title": "Test"})),
        Some(true),
    );

    let json = ctx.to_json().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["phase"], "post");
    assert_eq!(parsed["item_type"], "issue");
    assert_eq!(parsed["operation"], "create");
    assert_eq!(parsed["project_path"], "/tmp/project");
    assert_eq!(parsed["item_id"], "issue-123");
    assert_eq!(parsed["request_data"]["title"], "Test");
    assert_eq!(parsed["success"], true);
}
