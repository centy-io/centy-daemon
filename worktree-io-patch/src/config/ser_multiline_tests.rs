use super::*;

#[test]
fn test_multiline_hook_uses_triple_quotes() {
    let script = "#!/usr/bin/env bash\necho \"hello\"\n";
    let mut c = Config::default();
    c.hooks.post_open = Some(script.into());
    let s = c.to_toml_with_comments();
    assert!(
        s.contains("\"post:open\" = \"\"\"\n"),
        "expected multiline basic string"
    );
    assert!(!s.contains("\\n"), "newlines must not be escaped as \\n");
}

#[test]
fn test_multiline_hook_round_trips() {
    let script = "#!/usr/bin/env bash\necho \"Worktree ready\"\n";
    let mut c = Config::default();
    c.hooks.pre_open = Some(script.into());
    c.hooks.post_open = Some(script.into());
    let s = c.to_toml_with_comments();
    let parsed: Config = toml::from_str(&s).unwrap();
    assert_eq!(parsed.hooks.pre_open.as_deref(), Some(script));
    assert_eq!(parsed.hooks.post_open.as_deref(), Some(script));
}
