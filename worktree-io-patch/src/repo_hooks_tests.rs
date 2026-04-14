use super::*;

fn entry(script: &str, order: HookOrder) -> RepoHookEntry {
    RepoHookEntry {
        script: script.to_owned(),
        order,
    }
}

#[test]
fn test_combined_both_none() {
    assert!(combined_script(None, None).is_none());
}

#[test]
fn test_combined_global_only() {
    assert_eq!(combined_script(Some("global"), None).unwrap(), "global");
}

#[test]
fn test_combined_repo_only() {
    let e = entry("repo", HookOrder::Before);
    assert_eq!(combined_script(None, Some(&e)).unwrap(), "repo");
}

#[test]
fn test_combined_before() {
    let e = entry("repo", HookOrder::Before);
    assert_eq!(
        combined_script(Some("global"), Some(&e)).unwrap(),
        "repo\nglobal"
    );
}

#[test]
fn test_combined_after() {
    let e = entry("repo", HookOrder::After);
    assert_eq!(
        combined_script(Some("global"), Some(&e)).unwrap(),
        "global\nrepo"
    );
}

#[test]
fn test_combined_replace() {
    let e = entry("repo", HookOrder::Replace);
    assert_eq!(combined_script(Some("global"), Some(&e)).unwrap(), "repo");
}
