use super::*;
use tempfile::TempDir;

async fn create_test_issue(
    issues_path: &Path,
    issue_id: &str,
    display_number: u32,
    created_at: &str,
) {
    let frontmatter = format!(
        "---\ndisplayNumber: {display_number}\nstatus: open\npriority: 2\ncreatedAt: {created_at}\nupdatedAt: {created_at}\n---\n# Test Issue\n"
    );
    fs::write(issues_path.join(format!("{issue_id}.md")), frontmatter)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_reconcile_with_conflict() {
    let temp = TempDir::new().unwrap();
    let issues_path = temp.path().join("issues");
    fs::create_dir_all(&issues_path).await.unwrap();

    create_test_issue(
        &issues_path,
        "550e8400-e29b-41d4-a716-446655440001",
        4,
        "2024-01-01T10:00:00Z",
    )
    .await;
    create_test_issue(
        &issues_path,
        "550e8400-e29b-41d4-a716-446655440002",
        4,
        "2024-01-01T10:05:00Z",
    )
    .await;
    create_test_issue(
        &issues_path,
        "550e8400-e29b-41d4-a716-446655440003",
        5,
        "2024-01-01T10:10:00Z",
    )
    .await;

    let reassigned = reconcile_display_numbers(&issues_path).await.unwrap();
    assert_eq!(reassigned, 1);

    let content1 = fs::read_to_string(
        issues_path.join("550e8400-e29b-41d4-a716-446655440001.md"),
    )
    .await
    .unwrap();
    let (fm1, _, _): (IssueFrontmatter, String, String) =
        mdstore::parse_frontmatter(&content1).unwrap();
    assert_eq!(fm1.display_number, 4);

    let content2 = fs::read_to_string(
        issues_path.join("550e8400-e29b-41d4-a716-446655440002.md"),
    )
    .await
    .unwrap();
    let (fm2, _, _): (IssueFrontmatter, String, String) =
        mdstore::parse_frontmatter(&content2).unwrap();
    assert_eq!(fm2.display_number, 6);
}
