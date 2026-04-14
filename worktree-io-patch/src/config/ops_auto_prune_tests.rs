use super::*;

#[test]
fn test_get_value_workspace_auto_prune_default() {
    assert_eq!(
        Config::default().get_value("workspace.auto_prune").unwrap(),
        "false"
    );
}
#[test]
fn test_set_value_workspace_auto_prune_true() {
    let mut c = Config::default();
    c.set_value("workspace.auto_prune", "true").unwrap();
    assert!(c.workspace.auto_prune);
    assert_eq!(c.get_value("workspace.auto_prune").unwrap(), "true");
}
#[test]
fn test_set_value_workspace_auto_prune_false() {
    let mut c = Config::default();
    c.workspace.auto_prune = true;
    c.set_value("workspace.auto_prune", "false").unwrap();
    assert!(!c.workspace.auto_prune);
}
#[test]
fn test_set_value_workspace_auto_prune_invalid() {
    assert!(Config::default()
        .set_value("workspace.auto_prune", "not-a-bool")
        .is_err());
}
