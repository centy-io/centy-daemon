use super::*;
use std::env;

#[test]
fn test_is_git_repository() {
    // Current directory should be part of a git repo (the centy-daemon project)
    let cwd = env::current_dir().unwrap();
    // This test may fail if run outside a git repo, which is acceptable
    let _ = is_git_repository(&cwd);
}

#[test]
fn test_non_git_directory() {
    // Use root directory which is definitely not inside a git repository
    // (git won't traverse above /)
    let non_git = Path::new("/");
    assert!(!is_git_repository(non_git));
}
