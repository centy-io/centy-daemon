use super::{scaffold_if_missing, SCAFFOLD};
use crate::repo_hooks::RepoConfig;

#[test]
fn test_scaffold_creates_file_when_absent() {
    let dir = tempfile::tempdir().unwrap();
    let created = scaffold_if_missing(dir.path()).unwrap();
    assert!(created);
    let contents = std::fs::read_to_string(dir.path().join(".worktree.toml")).unwrap();
    assert_eq!(contents, SCAFFOLD);
}

#[test]
fn test_scaffold_skips_when_file_present() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".worktree.toml");
    std::fs::write(&path, b"existing").unwrap();
    let created = scaffold_if_missing(dir.path()).unwrap();
    assert!(!created);
    // existing content must be untouched
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "existing");
}

#[test]
fn test_scaffold_is_all_comments_or_blank() {
    for line in SCAFFOLD.lines() {
        let trimmed = line.trim();
        assert!(
            trimmed.is_empty() || trimmed.starts_with('#'),
            "unexpected non-comment line in scaffold: {line:?}"
        );
    }
}

#[test]
fn test_scaffold_parses_as_empty_config() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_if_missing(dir.path()).unwrap();
    let cfg = RepoConfig::load_from(dir.path()).unwrap();
    assert!(cfg.hooks.pre_open.is_none());
    assert!(cfg.hooks.post_open.is_none());
}
