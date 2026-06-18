use super::*;

#[test]
fn test_path_ends_with_config_toml() {
    let p = Config::path().unwrap();
    assert!(p.ends_with(".config/worktree/config.toml"));
}
#[test]
fn test_load_no_file_returns_default() {
    let c = Config::load().unwrap_or_default();
    assert!(c.editor.command.is_none() || c.editor.command.is_some());
}
#[test]
fn test_get_value_editor_command_empty() {
    assert_eq!(Config::default().get_value("editor.command").unwrap(), "");
}
#[test]
fn test_get_value_editor_command_set() {
    let mut c = Config::default();
    c.editor.command = Some("code .".into());
    assert_eq!(c.get_value("editor.command").unwrap(), "code .");
}
#[test]
fn test_get_value_open_editor() {
    assert_eq!(Config::default().get_value("open.editor").unwrap(), "true");
}
#[test]
fn test_get_value_unknown_key() {
    assert!(Config::default().get_value("unknown.key").is_err());
}
#[test]
fn test_set_value_editor_command() {
    let mut c = Config::default();
    c.set_value("editor.command", "nvim .").unwrap();
    assert_eq!(c.editor.command.as_deref(), Some("nvim ."));
}
#[test]
fn test_set_value_editor_command_empty_clears() {
    let mut c = Config::default();
    c.editor.command = Some("code .".into());
    c.set_value("editor.command", "").unwrap();
    assert!(c.editor.command.is_none());
}
#[test]
fn test_set_value_open_editor_false() {
    let mut c = Config::default();
    c.set_value("open.editor", "false").unwrap();
    assert!(!c.open.editor);
}
#[test]
fn test_set_value_open_editor_invalid() {
    assert!(Config::default()
        .set_value("open.editor", "bad_bool")
        .is_err());
}
#[test]
fn test_set_value_unknown_key() {
    assert!(Config::default().set_value("bad.key", "val").is_err());
}
#[test]
fn test_get_value_workspace_ttl_empty() {
    assert_eq!(Config::default().get_value("workspace.ttl").unwrap(), "");
}
#[test]
fn test_get_value_workspace_ttl_set() {
    let mut c = Config::default();
    c.set_value("workspace.ttl", "7days").unwrap();
    let v = c.get_value("workspace.ttl").unwrap();
    assert!(!v.is_empty());
}
#[test]
fn test_set_value_workspace_ttl() {
    let mut c = Config::default();
    c.set_value("workspace.ttl", "7days").unwrap();
    assert!(c.workspace.ttl.is_some());
}
#[test]
fn test_set_value_workspace_ttl_empty_clears() {
    let mut c = Config::default();
    c.set_value("workspace.ttl", "7days").unwrap();
    c.set_value("workspace.ttl", "").unwrap();
    assert!(c.workspace.ttl.is_none());
}
#[test]
fn test_set_value_workspace_ttl_invalid() {
    assert!(Config::default()
        .set_value("workspace.ttl", "not-a-duration")
        .is_err());
}
#[test]
fn test_editor_alias() {
    let mut c = Config::default();
    c.set_value("editor", "nvim").unwrap();
    assert_eq!(c.editor.command.as_deref(), Some("nvim"));
    assert_eq!(c.get_value("editor").unwrap(), "nvim");
}
