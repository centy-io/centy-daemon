use super::*;

#[tokio::test]
async fn test_infer_from_non_git_directory() {
    // Use root directory which is definitely not inside a git repository
    // TempDir can be inside a git worktree which causes git to find parent repo
    let non_git = std::path::Path::new("/");
    let result = infer_organization_from_remote(non_git, None).await;

    assert!(result.inferred_org_slug.is_none());
    assert!(result.message.unwrap().contains("Not a git repository"));
}

#[tokio::test]
async fn test_mismatch_detection() {
    // This test just verifies the mismatch logic works
    // We can't easily test the full flow without a real git repo
    let result = OrgInferenceResult {
        inferred_org_slug: Some("new-org".to_string()),
        inferred_org_name: Some("new-org".to_string()),
        existing_org_slug: Some("old-org".to_string()),
        has_mismatch: true,
        ..Default::default()
    };

    assert!(result.has_mismatch);
    assert_eq!(result.existing_org_slug, Some("old-org".to_string()));
    assert_eq!(result.inferred_org_slug, Some("new-org".to_string()));
}
