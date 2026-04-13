//! Tests verifying that hard-deleting an item also removes all of its link records.
#![allow(clippy::unwrap_used)]

use super::*;
use crate::config::item_type_config::default_issue_config;
use crate::config::CentyConfig;
use crate::link::{
    cascade_delete_entity_links, create_link, list_all_links, CreateLinkOptions, TargetType,
};
use std::collections::HashMap;

fn issue_config() -> TypeConfig {
    TypeConfig::from(&default_issue_config(&CentyConfig::default()))
}

async fn setup_project(temp: &std::path::Path) {
    let centy_path = temp.join(".centy");
    fs::create_dir_all(&centy_path).await.unwrap();
    let manifest = crate::manifest::create_manifest();
    crate::manifest::write_manifest(temp, &manifest)
        .await
        .unwrap();
}

async fn create_issue(temp: &std::path::Path, title: &str) -> mdstore::Item {
    let config = issue_config();
    let options = CreateOptions {
        title: title.to_string(),
        body: String::new(),
        id: None,
        status: Some("open".to_string()),
        priority: Some(2),
        tags: None,
        custom_fields: HashMap::new(),
        comment: None,
    };
    generic_create(temp, "issues", &config, options).await.unwrap()
}

#[tokio::test]
async fn test_hard_delete_item_cascades_links() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let a = create_issue(temp.path(), "Issue A").await;
    let b = create_issue(temp.path(), "Issue B").await;

    // Create two links involving A
    create_link(
        temp.path(),
        CreateLinkOptions {
            source_id: a.id.clone(),
            source_type: TargetType::issue(),
            target_id: b.id.clone(),
            target_type: TargetType::issue(),
            link_type: "blocks".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    let c = create_issue(temp.path(), "Issue C").await;
    create_link(
        temp.path(),
        CreateLinkOptions {
            source_id: c.id.clone(),
            source_type: TargetType::issue(),
            target_id: a.id.clone(),
            target_type: TargetType::issue(),
            link_type: "relates-to".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    // Sanity check: two links exist
    let links_before = list_all_links(temp.path()).await.unwrap();
    assert_eq!(links_before.len(), 2);

    // Hard-delete issue A — should cascade-delete both links
    generic_delete(temp.path(), "issues", &issue_config(), &a.id, true)
        .await
        .unwrap();

    let links_after = list_all_links(temp.path()).await.unwrap();
    assert!(
        links_after.is_empty(),
        "All links referencing deleted item should be removed, got: {links_after:?}"
    );
}

#[tokio::test]
async fn test_hard_delete_item_preserves_unrelated_links() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let a = create_issue(temp.path(), "Issue A").await;
    let b = create_issue(temp.path(), "Issue B").await;
    let c = create_issue(temp.path(), "Issue C").await;

    // A → B (will be deleted when A is deleted)
    create_link(
        temp.path(),
        CreateLinkOptions {
            source_id: a.id.clone(),
            source_type: TargetType::issue(),
            target_id: b.id.clone(),
            target_type: TargetType::issue(),
            link_type: "blocks".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    // B → C (unrelated to A — must survive A's deletion)
    create_link(
        temp.path(),
        CreateLinkOptions {
            source_id: b.id.clone(),
            source_type: TargetType::issue(),
            target_id: c.id.clone(),
            target_type: TargetType::issue(),
            link_type: "relates-to".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    generic_delete(temp.path(), "issues", &issue_config(), &a.id, true)
        .await
        .unwrap();

    let links_after = list_all_links(temp.path()).await.unwrap();
    assert_eq!(links_after.len(), 1, "Unrelated link B->C should be preserved");
    assert_eq!(links_after[0].source_id, b.id);
    assert_eq!(links_after[0].target_id, c.id);
}

#[tokio::test]
async fn test_soft_delete_does_not_cascade_links() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let a = create_issue(temp.path(), "Issue A").await;
    let b = create_issue(temp.path(), "Issue B").await;

    create_link(
        temp.path(),
        CreateLinkOptions {
            source_id: a.id.clone(),
            source_type: TargetType::issue(),
            target_id: b.id.clone(),
            target_type: TargetType::issue(),
            link_type: "blocks".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    // Soft delete (force=false) must NOT touch links
    generic_soft_delete(temp.path(), "issues", &a.id)
        .await
        .unwrap();

    let links_after = list_all_links(temp.path()).await.unwrap();
    assert_eq!(
        links_after.len(),
        1,
        "Soft delete must not cascade-delete links"
    );
}

#[tokio::test]
async fn test_cascade_delete_entity_links_public_api() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join(".centy")).await.unwrap();
    crate::manifest::write_manifest(temp.path(), &crate::manifest::create_manifest())
        .await
        .unwrap();

    let a = create_issue(temp.path(), "Issue A").await;
    let b = create_issue(temp.path(), "Issue B").await;

    create_link(
        temp.path(),
        CreateLinkOptions {
            source_id: a.id.clone(),
            source_type: TargetType::issue(),
            target_id: b.id.clone(),
            target_type: TargetType::issue(),
            link_type: "blocks".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    let deleted = cascade_delete_entity_links(temp.path(), &a.id)
        .await
        .unwrap();
    assert_eq!(deleted, 1);

    let links = list_all_links(temp.path()).await.unwrap();
    assert!(links.is_empty());
}
