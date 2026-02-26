use super::*;
use tempfile::TempDir;

async fn create_test_issue_2(
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
async fn test_get_next_display_number_empty() {
    let temp = TempDir::new().unwrap();
    let issues_path = temp.path().join("issues");

    let next = get_next_display_number(&issues_path).await.unwrap();
    assert_eq!(next, 1);
}

#[tokio::test]
async fn test_get_next_display_number_with_existing() {
    let temp = TempDir::new().unwrap();
    let issues_path = temp.path().join("issues");
    fs::create_dir_all(&issues_path).await.unwrap();

    create_test_issue_2(
        &issues_path,
        "550e8400-e29b-41d4-a716-446655440001",
        5,
        "2024-01-01T10:00:00Z",
    )
    .await;

    let next = get_next_display_number(&issues_path).await.unwrap();
    assert_eq!(next, 6);
}
