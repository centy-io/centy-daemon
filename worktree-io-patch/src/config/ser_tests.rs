use super::*;

#[test]
fn test_website_comment_is_first_line() {
    let s = Config::default().to_toml_with_comments();
    assert!(s.starts_with("# runner \u{2014} https://worktree.io\n"));
}

#[test]
fn test_default_output_structure() {
    let s = Config::default().to_toml_with_comments();
    assert!(s.contains("[editor]"));
    assert!(s.contains("[open]"));
    assert!(s.contains("[hooks]"));
    assert!(s.contains("[workspace]"));
    assert!(s.contains("# Editor configuration."));
    assert!(s.contains("# Workspace open behavior."));
    assert!(s.contains("# Hook scripts run around the open command."));
    assert!(s.contains("# Whether to launch the configured editor when opening a workspace."));
    assert!(s.contains("editor = true"));
    assert!(!s.contains("command ="));
    assert!(!s.contains("pre:open"));
    assert!(!s.contains("post:open"));
}

#[test]
fn test_command_field_with_comment() {
    let mut c = Config::default();
    c.editor.command = Some("code .".into());
    let s = c.to_toml_with_comments();
    assert!(s.contains("# Command to launch the editor"));
    assert!(s.contains("command = \"code .\""));
}

#[test]
fn test_hooks_with_comments() {
    let mut c = Config::default();
    c.hooks.pre_open = Some("echo pre".into());
    c.hooks.post_open = Some("echo post".into());
    let s = c.to_toml_with_comments();
    assert!(s.contains("# Script run before opening the workspace."));
    assert!(s.contains("\"pre:open\" = \"echo pre\""));
    assert!(s.contains("# Script run after opening the workspace."));
    assert!(s.contains("\"post:open\" = \"echo post\""));
}

#[test]
fn test_round_trips_default() {
    let c = Config::default();
    let s = c.to_toml_with_comments();
    let parsed: Config = toml::from_str(&s).unwrap();
    assert!(parsed.editor.command.is_none());
    assert!(parsed.open.editor);
    assert!(parsed.hooks.pre_open.is_none());
    assert!(parsed.hooks.post_open.is_none());
}

#[test]
fn test_round_trips_all_fields() {
    let mut c = Config::default();
    c.editor.command = Some("nvim .".into());
    c.open.editor = false;
    c.hooks.pre_open = Some("echo pre".into());
    c.hooks.post_open = Some("echo post".into());
    let s = c.to_toml_with_comments();
    let parsed: Config = toml::from_str(&s).unwrap();
    assert_eq!(parsed.editor.command.as_deref(), Some("nvim ."));
    assert!(!parsed.open.editor);
    assert_eq!(parsed.hooks.pre_open.as_deref(), Some("echo pre"));
    assert_eq!(parsed.hooks.post_open.as_deref(), Some("echo post"));
}

#[test]
fn test_escapes_special_chars() {
    let mut c = Config::default();
    c.editor.command = Some(r#"cmd "with" quotes"#.into());
    let s = c.to_toml_with_comments();
    let parsed: Config = toml::from_str(&s).unwrap();
    assert_eq!(
        parsed.editor.command.as_deref(),
        Some(r#"cmd "with" quotes"#)
    );
}

#[test]
fn test_background_serialization() {
    assert!(!Config::default()
        .to_toml_with_comments()
        .contains("background ="));
    let mut c = Config::default();
    c.editor.background = true;
    let s = c.to_toml_with_comments();
    assert!(s.contains("background = true"));
    assert!(s.contains("# When true, the editor opens in the background"));
    let parsed: Config = toml::from_str(&s).unwrap();
    assert!(parsed.editor.background);
}
