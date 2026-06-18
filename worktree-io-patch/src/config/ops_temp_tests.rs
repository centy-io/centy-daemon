use super::*;

#[test]
fn test_get_value_workspace_temp_default() {
    assert_eq!(
        Config::default().get_value("workspace.temp").unwrap(),
        "false"
    );
}
#[test]
fn test_set_value_workspace_temp_true() {
    let mut c = Config::default();
    c.set_value("workspace.temp", "true").unwrap();
    assert!(c.workspace.temp);
    assert_eq!(c.get_value("workspace.temp").unwrap(), "true");
}
#[test]
fn test_set_value_workspace_temp_false() {
    let mut c = Config::default();
    c.workspace.temp = true;
    c.set_value("workspace.temp", "false").unwrap();
    assert!(!c.workspace.temp);
}
#[test]
fn test_set_value_workspace_temp_invalid() {
    assert!(Config::default()
        .set_value("workspace.temp", "not-a-bool")
        .is_err());
}
