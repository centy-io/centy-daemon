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
async fn test_reconcile_no_conflicts() {
    let temp = TempDir::new().unwrap();
    let issues_path = temp.path().join("issues");
    fs::create_dir_all(&issues_path).await.unwrap();

    create_test_issue(
        &issues_path,
        "550e8400-e29b-41d4-a716-446655440001",
        1,
        "2024-01-01T10:00:00Z",
    )
    .await;
    create_test_issue(
        &issues_path,
        "550e8400-e29b-41d4-a716-446655440002",
        2,
        "2024-01-01T11:00:00Z",
    )
    .await;

    let reassigned = reconcile_display_numbers(&issues_path).await.unwrap();
    assert_eq!(reassigned, 0);
}
