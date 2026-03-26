use super::super::types::TargetType;
use super::{create_link_file, delete_link_file, list_all_link_records};

#[tokio::test]
async fn test_delete_link_file_removes_record() {
    use tempfile::tempdir;
    let temp = tempdir().unwrap();
    let centy = temp.path().join(".centy");
    tokio::fs::create_dir_all(&centy).await.unwrap();

    let record = create_link_file(
        temp.path(),
        "a",
        &TargetType::issue(),
        "b",
        &TargetType::issue(),
        "blocks",
    )
    .await
    .unwrap();

    delete_link_file(temp.path(), &record.id).await.unwrap();

    let records = list_all_link_records(temp.path()).await.unwrap();
    assert!(records.is_empty());
}

#[tokio::test]
async fn test_delete_nonexistent_link_returns_error() {
    use tempfile::tempdir;
    let temp = tempdir().unwrap();
    let centy = temp.path().join(".centy");
    tokio::fs::create_dir_all(&centy).await.unwrap();

    let result = delete_link_file(temp.path(), "nonexistent-uuid").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_multiple_links_stored_and_listed() {
    use tempfile::tempdir;
    let temp = tempdir().unwrap();
    let centy = temp.path().join(".centy");
    tokio::fs::create_dir_all(&centy).await.unwrap();

    create_link_file(
        temp.path(),
        "a",
        &TargetType::issue(),
        "b",
        &TargetType::issue(),
        "blocks",
    )
    .await
    .unwrap();
    create_link_file(
        temp.path(),
        "a",
        &TargetType::issue(),
        "c",
        &TargetType::issue(),
        "relates-to",
    )
    .await
    .unwrap();

    let records = list_all_link_records(temp.path()).await.unwrap();
    assert_eq!(records.len(), 2);
}
