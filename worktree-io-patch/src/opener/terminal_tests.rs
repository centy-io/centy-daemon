use super::*;
#[test]
fn test_ide_command_returns_false() {
    let p = std::path::Path::new("/tmp");
    assert!(!try_terminal_with_init(p, "code .", "echo hi").unwrap());
}
#[test]
fn test_unknown_command_returns_false() {
    let p = std::path::Path::new("/tmp");
    assert!(!try_terminal_with_init(p, "hx .", "echo hi").unwrap());
}
#[test]
fn test_tmux_returns_true() {
    let p = std::path::Path::new("/tmp");
    assert!(try_terminal_with_init(p, "tmux", "echo hi").unwrap());
}
#[test]
fn test_open_editor_or_terminal_tmux() {
    let p = std::path::Path::new("/tmp");
    super::super::open_editor_or_terminal(p, "tmux", false).unwrap();
}
