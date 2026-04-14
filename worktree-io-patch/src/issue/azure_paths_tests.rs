use super::*;

#[test]
fn test_parse_azure_devops_worktree_url_with_editor() {
    let (r, opts) = IssueRef::parse_with_options(
        "worktree://open?org=myorg&project=myproject&work_item_id=42&editor=cursor",
    )
    .unwrap();
    assert_eq!(
        r,
        IssueRef::AzureDevOps {
            org: "myorg".into(),
            project: "myproject".into(),
            repo: "myproject".into(),
            id: 42,
        }
    );
    assert_eq!(opts.editor.as_deref(), Some("cursor"));
}

#[test]
fn test_azure_devops_workspace_dir_name() {
    let r = IssueRef::AzureDevOps {
        org: "myorg".into(),
        project: "myproject".into(),
        repo: "myrepo".into(),
        id: 42,
    };
    assert_eq!(r.workspace_dir_name(), "workitem-42");
    assert_eq!(r.branch_name(), "workitem-42");
}

#[test]
fn test_azure_devops_clone_url() {
    let r = IssueRef::AzureDevOps {
        org: "myorg".into(),
        project: "myproject".into(),
        repo: "myrepo".into(),
        id: 42,
    };
    assert_eq!(
        r.clone_url(),
        "https://dev.azure.com/myorg/myproject/_git/myrepo"
    );
}

#[test]
fn test_azure_devops_paths() {
    let r = IssueRef::AzureDevOps {
        org: "myorg".into(),
        project: "myproject".into(),
        repo: "myrepo".into(),
        id: 42,
    };
    assert!(r
        .bare_clone_path()
        .ends_with("worktrees/azuredevops/myorg/myproject/myrepo"));
    assert!(r
        .temp_path()
        .ends_with("worktrees/azuredevops/myorg/myproject/myrepo/workitem-42"));
}
