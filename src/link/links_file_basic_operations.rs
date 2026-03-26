use super::super::types::{LinkDirection, LinkRecord, TargetType};
use super::{create_link_file, list_all_link_records};

#[tokio::test]
async fn test_create_link_file_returns_record() {
    use tempfile::tempdir;
    let temp = tempdir().unwrap();
    // Create stub entity files so entity_exists checks work (not needed for storage tests).
    let centy = temp.path().join(".centy");
    tokio::fs::create_dir_all(&centy).await.unwrap();

    let record = create_link_file(
        temp.path(),
        "src-uuid",
        &TargetType::issue(),
        "tgt-uuid",
        &TargetType::new("doc"),
        "blocks",
    )
    .await
    .unwrap();

    assert!(!record.id.is_empty());
    assert_eq!(record.source_id, "src-uuid");
    assert_eq!(record.source_type, TargetType::issue());
    assert_eq!(record.target_id, "tgt-uuid");
    assert_eq!(record.target_type, TargetType::new("doc"));
    assert_eq!(record.link_type, "blocks");
    assert!(!record.created_at.is_empty());
}

#[tokio::test]
async fn test_list_all_link_records_empty() {
    use tempfile::tempdir;
    let temp = tempdir().unwrap();
    let records = list_all_link_records(temp.path()).await.unwrap();
    assert!(records.is_empty());
}

#[tokio::test]
async fn test_list_all_link_records_returns_created() {
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

    let records = list_all_link_records(temp.path()).await.unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].source_id, "a");
    assert_eq!(records[0].target_id, "b");
    assert_eq!(records[0].link_type, "blocks");
}

#[test]
fn test_source_view_direction() {
    let record = LinkRecord {
        id: "id".to_string(),
        source_id: "s".to_string(),
        source_type: TargetType::issue(),
        target_id: "t".to_string(),
        target_type: TargetType::issue(),
        link_type: "blocks".to_string(),
        created_at: "ts".to_string(),
        updated_at: "ts".to_string(),
    };
    assert_eq!(record.source_view().direction, LinkDirection::Source);
    assert_eq!(record.target_view().direction, LinkDirection::Target);
}
