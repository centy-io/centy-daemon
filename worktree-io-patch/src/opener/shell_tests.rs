use super::*;
#[test]
fn test_shlex_simple() {
    assert_eq!(
        shlex_split("git commit -m init"),
        vec!["git", "commit", "-m", "init"]
    );
}
#[test]
fn test_shlex_quoted() {
    assert_eq!(
        shlex_split(r#"git commit -m "hello world""#),
        vec!["git", "commit", "-m", "hello world"]
    );
}
#[test]
fn test_shlex_tabs() {
    assert_eq!(shlex_split("a\tb"), vec!["a", "b"]);
}
#[test]
fn test_shlex_empty() {
    assert!(shlex_split("").is_empty());
}
#[cfg(not(windows))]
#[test]
fn test_augmented_path_contains_homebrew() {
    let p = augmented_path();
    assert!(p.contains("/opt/homebrew/bin"));
}
#[cfg(windows)]
#[test]
fn test_augmented_path_on_windows_returns_current() {
    let current = std::env::var("PATH").unwrap_or_default();
    assert_eq!(augmented_path(), current);
}
#[test]
fn test_run_shell_command_empty() {
    assert!(run_shell_command("", false).is_err());
}
#[cfg(not(windows))]
#[test]
fn test_run_shell_command_success() {
    run_shell_command("true", false).unwrap();
    run_shell_command("true", true).unwrap();
}
#[cfg(windows)]
#[test]
fn test_run_shell_command_success_windows() {
    run_shell_command("cmd /C echo hello", false).unwrap();
    run_shell_command("cmd /C echo hello", true).unwrap();
}
#[test]
fn test_run_shell_command_bad_program() {
    assert!(run_shell_command("__nonexistent_xyz_wt__", true).is_err());
    assert!(run_shell_command("__nonexistent_xyz_wt__", false).is_err());
}
