use super::*;
use tempfile::TempDir;

async fn create_test_issue(
    issues_path: &Path,
    folder_name: &str,
    display_number: u32,
    created_at: &str,
) {
    let issue_path = issues_path.join(folder_name);
    fs::create_dir_all(&issue_path).await.unwrap();

    let metadata = serde_json::json!({
        "displayNumber": display_number,
        "status": "open",
        "priority": 2,
        "createdAt": created_at,
        "updatedAt": created_at
    });

    fs::write(
        issue_path.join("metadata.json"),
        serde_json::to_string_pretty(&metadata).unwrap(),
    )
    .await
    .unwrap();

    fs::write(issue_path.join("issue.md"), "# Test Issue\n")
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

    let metadata1: IssueMetadata = serde_json::from_str(
        &fs::read_to_string(
            issues_path
                .join("550e8400-e29b-41d4-a716-446655440001")
                .join("metadata.json"),
        )
        .await
        .unwrap(),
    )
    .unwrap();
    assert_eq!(metadata1.common.display_number, 4);

    let metadata2: IssueMetadata = serde_json::from_str(
        &fs::read_to_string(
            issues_path
                .join("550e8400-e29b-41d4-a716-446655440002")
                .join("metadata.json"),
        )
        .await
        .unwrap(),
    )
    .unwrap();
    assert_eq!(metadata2.common.display_number, 6);
}
