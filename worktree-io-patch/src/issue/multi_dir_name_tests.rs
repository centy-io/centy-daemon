use super::IssueRef;
use std::path::PathBuf;

#[test]
#[should_panic(expected = "multi_dir_name is not supported for IssueRef::Local")]
fn local_panics() {
    let r = IssueRef::Local {
        project_path: PathBuf::from("/tmp/proj"),
        display_number: 1,
    };
    let _ = r.multi_dir_name();
}

#[test]
fn github_repo_and_number() {
    let r = IssueRef::GitHub {
        owner: "acme".into(),
        repo: "backend".into(),
        number: 7,
    };
    assert_eq!(r.multi_dir_name(), "backend-7");
}

#[test]
fn gitlab_repo_and_number() {
    let r = IssueRef::GitLab {
        owner: "acme".into(),
        repo: "api".into(),
        number: 12,
    };
    assert_eq!(r.multi_dir_name(), "api-12");
}

#[test]
fn linear_repo_and_id() {
    let r = IssueRef::Linear {
        owner: "acme".into(),
        repo: "backend".into(),
        id: "abc-123".into(),
    };
    assert_eq!(r.multi_dir_name(), "backend-abc-123");
}

#[test]
fn azure_repo_and_id() {
    let r = IssueRef::AzureDevOps {
        org: "myorg".into(),
        project: "myproj".into(),
        repo: "service".into(),
        id: 99,
    };
    assert_eq!(r.multi_dir_name(), "service-99");
}

#[test]
fn jira_repo_and_key_lowercased() {
    let r = IssueRef::Jira {
        host: "acme.atlassian.net".into(),
        issue_key: "PROJ-42".into(),
        owner: "acme".into(),
        repo: "backend".into(),
    };
    assert_eq!(r.multi_dir_name(), "backend-proj-42");
}
