use super::*;
#[cfg(target_os = "macos")]
#[test]
fn test_app_exists_nonexistent() {
    assert!(!app_exists("__NoSuchApp__"));
}
#[test]
fn test_open_with_hook_ide_no_terminal() {
    let p = std::path::Path::new("/tmp");
    // "code ." is not a terminal, and no /Applications/iTerm.app etc in CI
    let _ = open_with_hook(p, "echo .", "true", false);
}
#[test]
fn test_open_in_editor_dot_substitution() {
    let p = std::path::Path::new("/tmp/myproject");
    open_in_editor(p, "echo .", false).unwrap();
}
#[test]
fn test_open_in_editor_no_dot() {
    let p = std::path::Path::new("/tmp/myproject");
    open_in_editor(p, "echo", false).unwrap();
}
#[test]
fn test_open_in_editor_background() {
    let p = std::path::Path::new("/tmp/myproject");
    open_in_editor(p, "echo", true).unwrap();
}
#[test]
fn test_open_editor_or_terminal_ide() {
    let p = std::path::Path::new("/tmp/myproject");
    open_editor_or_terminal(p, "echo .", false).unwrap();
}
