use super::*;

fn ctx() -> HookContext {
    HookContext {
        owner: "acme".into(),
        repo: "api".into(),
        issue: "42".into(),
        branch: "issue-42".into(),
        worktree_path: "/tmp/wt".into(),
    }
}

/// Regression: multi-line hook must execute every line, not just the first.
/// A `sh -c "script"` approach collapses newlines to literal `\n`, so only
/// the shebang/first line runs. The temp-file approach must be used instead.
#[cfg(not(windows))]
#[test]
fn test_run_hook_multiline_executes_all_lines() {
    let out = std::env::temp_dir().join("worktree-test-multiline-hook.txt");
    let _ = std::fs::remove_file(&out);
    let path = out.to_str().unwrap().to_string();
    let script = format!("printf 'first' > '{path}'\nprintf 'second' >> '{path}'\n");
    run_hook(&script, &ctx()).unwrap();
    let content = std::fs::read_to_string(&out).unwrap_or_default();
    let _ = std::fs::remove_file(&out);
    assert_eq!(
        content, "firstsecond",
        "all lines of a multi-line hook must run"
    );
}
