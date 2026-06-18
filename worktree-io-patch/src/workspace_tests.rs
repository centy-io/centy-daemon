use super::*;
use crate::issue::IssueRef;

#[test]
fn test_open_or_create_existing() {
    let issue = IssueRef::GitHub {
        owner: "__test_wt__".into(),
        repo: "__test_wt__".into(),
        number: 9999,
    };
    let path = issue.temp_path();
    std::fs::create_dir_all(&path).unwrap();
    let ws = Workspace::open_or_create(issue).unwrap();
    assert!(!ws.created);
    std::fs::remove_dir_all(&path).ok();
}
