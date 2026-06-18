use super::*;
use std::path::PathBuf;

#[test]
fn test_local_workspace_dir_name() {
    let r = IssueRef::Local {
        project_path: PathBuf::from("/tmp/proj"),
        display_number: 5,
    };
    assert_eq!(r.workspace_dir_name(), "issue-5");
}

#[test]
fn test_local_branch_name() {
    let r = IssueRef::Local {
        project_path: PathBuf::from("/tmp/proj"),
        display_number: 7,
    };
    assert_eq!(r.branch_name(), "issue-7");
}

#[test]
fn test_local_paths() {
    let r = IssueRef::Local {
        project_path: PathBuf::from("/tmp/myproject"),
        display_number: 3,
    };
    assert!(r.bare_clone_path().ends_with("worktrees/local/myproject"));
    assert!(r.temp_path().ends_with("worktrees/local/myproject/issue-3"));
}

#[test]
#[should_panic(expected = "clone_url is never called")]
fn test_local_clone_url_panics() {
    let r = IssueRef::Local {
        project_path: PathBuf::from("/tmp/proj"),
        display_number: 1,
    };
    let _ = r.clone_url();
}
