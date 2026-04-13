//! Tests for link/storage/io.rs covering all branches.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{create_link_file, delete_link_file, delete_links_for_entity, list_all_link_records};
use crate::link::TargetType;

// ─── create_link_file ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_link_file_stores_all_fields() {
    let temp = tempfile::tempdir().unwrap();
    let centy = temp.path().join(".centy");
    tokio::fs::create_dir_all(&centy).await.unwrap();

    let record = create_link_file(
        temp.path(),
        "src-uuid",
        &TargetType::issue(),
        "tgt-uuid",
        &TargetType::new("doc"),
        "relates-to",
    )
    .await
    .unwrap();

    assert!(!record.id.is_empty());
    assert_eq!(record.source_id, "src-uuid");
    assert_eq!(record.source_type, TargetType::issue());
    assert_eq!(record.target_id, "tgt-uuid");
    assert_eq!(record.target_type, TargetType::new("doc"));
    assert_eq!(record.link_type, "relates-to");
    assert!(!record.created_at.is_empty());
    assert!(!record.updated_at.is_empty());
}

#[tokio::test]
async fn test_create_link_file_with_custom_type() {
    let temp = tempfile::tempdir().unwrap();
    tokio::fs::create_dir_all(temp.path().join(".centy"))
        .await
        .unwrap();

    let record = create_link_file(
        temp.path(),
        "a",
        &TargetType::issue(),
        "b",
        &TargetType::issue(),
        "custom-type",
    )
    .await
    .unwrap();

    assert_eq!(record.link_type, "custom-type");
    assert_eq!(record.source_type, TargetType::issue());
    assert_eq!(record.target_type, TargetType::issue());
}

// ─── list_all_link_records ───────────────────────────────────────────────────

#[tokio::test]
async fn test_list_all_link_records_no_centy_dir_returns_empty() {
    let temp = tempfile::tempdir().unwrap();
    // No .centy directory — links dir doesn't exist
    let records = list_all_link_records(temp.path()).await.unwrap();
    assert!(records.is_empty());
}

#[tokio::test]
async fn test_list_all_link_records_preserves_types() {
    let temp = tempfile::tempdir().unwrap();
    tokio::fs::create_dir_all(temp.path().join(".centy"))
        .await
        .unwrap();

    create_link_file(
        temp.path(),
        "issue-id",
        &TargetType::issue(),
        "doc-id",
        &TargetType::new("doc"),
        "parent-of",
    )
    .await
    .unwrap();

    let records = list_all_link_records(temp.path()).await.unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].source_type, TargetType::issue());
    assert_eq!(records[0].target_type, TargetType::new("doc"));
    assert_eq!(records[0].link_type, "parent-of");
}

// ─── delete_link_file ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_link_file_specific_record() {
    let temp = tempfile::tempdir().unwrap();
    tokio::fs::create_dir_all(temp.path().join(".centy"))
        .await
        .unwrap();

    let r1 = create_link_file(
        temp.path(),
        "a",
        &TargetType::issue(),
        "b",
        &TargetType::issue(),
        "blocks",
    )
    .await
    .unwrap();

    let r2 = create_link_file(
        temp.path(),
        "a",
        &TargetType::issue(),
        "c",
        &TargetType::issue(),
        "relates-to",
    )
    .await
    .unwrap();

    // Delete only the first record
    delete_link_file(temp.path(), &r1.id).await.unwrap();

    let records = list_all_link_records(temp.path()).await.unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, r2.id);
    assert_eq!(records[0].link_type, "relates-to");
}

// ─── delete_links_for_entity ─────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_links_for_entity_as_source() {
    let temp = tempfile::tempdir().unwrap();
    tokio::fs::create_dir_all(temp.path().join(".centy"))
        .await
        .unwrap();

    // entity "a" is source in both links
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
    // unrelated link
    create_link_file(
        temp.path(),
        "b",
        &TargetType::issue(),
        "c",
        &TargetType::issue(),
        "blocks",
    )
    .await
    .unwrap();

    let deleted = delete_links_for_entity(temp.path(), "a").await.unwrap();
    assert_eq!(deleted, 2);

    let remaining = list_all_link_records(temp.path()).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].source_id, "b");
    assert_eq!(remaining[0].target_id, "c");
}

#[tokio::test]
async fn test_delete_links_for_entity_as_target() {
    let temp = tempfile::tempdir().unwrap();
    tokio::fs::create_dir_all(temp.path().join(".centy"))
        .await
        .unwrap();

    // entity "z" is target in one link
    create_link_file(
        temp.path(),
        "x",
        &TargetType::issue(),
        "z",
        &TargetType::issue(),
        "blocks",
    )
    .await
    .unwrap();
    // unrelated
    create_link_file(
        temp.path(),
        "x",
        &TargetType::issue(),
        "y",
        &TargetType::issue(),
        "relates-to",
    )
    .await
    .unwrap();

    let deleted = delete_links_for_entity(temp.path(), "z").await.unwrap();
    assert_eq!(deleted, 1);

    let remaining = list_all_link_records(temp.path()).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].target_id, "y");
}

#[tokio::test]
async fn test_delete_links_for_entity_no_links() {
    let temp = tempfile::tempdir().unwrap();
    tokio::fs::create_dir_all(temp.path().join(".centy"))
        .await
        .unwrap();

    let deleted = delete_links_for_entity(temp.path(), "ghost-id").await.unwrap();
    assert_eq!(deleted, 0);
}
