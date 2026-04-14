use super::*;

#[test]
fn test_config_default() {
    let c = Config::default();
    assert!(c.editor.command.is_none());
    assert!(c.open.editor);
    assert!(c.hooks.pre_open.is_none());
}

#[test]
fn test_editor_config_default() {
    let e = EditorConfig::default();
    assert!(e.command.is_none());
    assert!(!e.background);
}

#[test]
fn test_open_config_default() {
    assert!(OpenConfig::default().editor);
}

#[test]
fn test_editor_background_get_set() {
    assert_eq!(
        Config::default().get_value("editor.background").unwrap(),
        "false"
    );
    let mut c = Config::default();
    c.set_value("editor.background", "true").unwrap();
    assert!(c.editor.background);
}

#[test]
fn test_editor_background_set_invalid() {
    assert!(Config::default()
        .set_value("editor.background", "bad")
        .is_err());
}
