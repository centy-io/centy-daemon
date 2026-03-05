use super::*;
use crate::config::item_type_config::{default_issue_config, write_item_type_config};
use crate::config::CentyConfig;
use crate::item::generic::storage::{generic_create, generic_list, generic_soft_delete};
use crate::manifest;
use chrono::Duration;
use mdstore::{CreateOptions, Filters, TypeConfig};
use std::collections::HashMap;
use tokio::fs;

// ─── parse_retention_duration ────────────────────────────────────────────────

#[test]
fn test_parse_days() {
    assert_eq!(parse_retention_duration("30d"), Some(Duration::days(30)));
    assert_eq!(parse_retention_duration("7d"), Some(Duration::days(7)));
    assert_eq!(parse_retention_duration("1d"), Some(Duration::days(1)));
}

#[test]
fn test_parse_hours() {
    assert_eq!(parse_retention_duration("24h"), Some(Duration::hours(24)));
    assert_eq!(parse_retention_duration("1h"), Some(Duration::hours(1)));
}

#[test]
fn test_parse_disabled_values() {
    assert_eq!(parse_retention_duration("0"), None);
    assert_eq!(parse_retention_duration(""), None);
    assert_eq!(parse_retention_duration("  "), None);
}

#[test]
fn test_parse_invalid() {
    assert_eq!(parse_retention_duration("abc"), None);
    assert_eq!(parse_retention_duration("30m"), None);
    assert_eq!(parse_retention_duration("-1d"), None);
    assert_eq!(parse_retention_duration("0d"), None);
    assert_eq!(parse_retention_duration("0h"), None);
}

// ─── run_cleanup_for_project ──────────────────────────────────────────────────

async fn setup_project(temp: &std::path::Path) {
    let centy_path = temp.join(".centy");
    fs::create_dir_all(&centy_path).await.unwrap();
    let manifest = manifest::create_manifest();
    manifest::write_manifest(temp, &manifest).await.unwrap();
}

fn issue_config() -> TypeConfig {
    TypeConfig::from(&default_issue_config(&CentyConfig::default()))
}

async fn write_issue_type_config(project_path: &std::path::Path) {
    let itc = default_issue_config(&CentyConfig::default());
    write_item_type_config(project_path, "issues", &itc)
        .await
        .unwrap();
}

async fn create_issue(project_path: &std::path::Path, title: &str) -> mdstore::Item {
    let options = CreateOptions {
        title: title.to_string(),
        body: String::new(),
        id: None,
        status: Some("open".to_string()),
        priority: Some(2),
        custom_fields: HashMap::new(),
    };
    generic_create(project_path, "issues", &issue_config(), options)
        .await
        .unwrap()
}

/// Backdate the `deleted_at` field in an item's frontmatter on disk so tests
/// can simulate items that have been deleted in the past.
async fn backdate_deleted_at(
    project_path: &std::path::Path,
    id: &str,
    deleted_at: &chrono::DateTime<chrono::Utc>,
) {
    let file_path = project_path
        .join(".centy")
        .join("issues")
        .join(format!("{id}.md"));
    let content = fs::read_to_string(&file_path).await.unwrap();
    // Replace the deletedAt line in the frontmatter
    let new_ts = deleted_at.to_rfc3339();
    let updated = content
        .lines()
        .map(|line| {
            if line.starts_with("deletedAt:") {
                format!("deletedAt: {new_ts}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&file_path, updated).await.unwrap();
}

#[tokio::test]
async fn test_expired_artifact_is_hard_deleted() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    write_issue_type_config(temp.path()).await;

    let item = create_issue(temp.path(), "Old deleted issue").await;
    generic_soft_delete(temp.path(), "issues", &item.id)
        .await
        .unwrap();

    // Backdate deleted_at to 40 days ago (beyond default 30d retention)
    let old_ts = chrono::Utc::now() - Duration::days(40);
    backdate_deleted_at(temp.path(), &item.id, &old_ts).await;

    run_cleanup_for_project(temp.path(), Duration::days(30)).await;

    // Item should be gone even when including deleted
    let all_items = generic_list(temp.path(), "issues", Filters::new().include_deleted())
        .await
        .unwrap();
    assert!(
        all_items.iter().all(|i| i.id != item.id),
        "Expired item should have been hard-deleted"
    );
}

#[tokio::test]
async fn test_non_expired_artifact_is_kept() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    write_issue_type_config(temp.path()).await;

    let item = create_issue(temp.path(), "Recent deleted issue").await;
    generic_soft_delete(temp.path(), "issues", &item.id)
        .await
        .unwrap();

    // deleted_at is "now" — well within the 30-day retention
    run_cleanup_for_project(temp.path(), Duration::days(30)).await;

    let all_items = generic_list(temp.path(), "issues", Filters::new().include_deleted())
        .await
        .unwrap();
    assert!(
        all_items.iter().any(|i| i.id == item.id),
        "Non-expired item should not have been deleted"
    );
}

#[tokio::test]
async fn test_non_deleted_item_untouched() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    write_issue_type_config(temp.path()).await;

    let item = create_issue(temp.path(), "Active issue").await;

    run_cleanup_for_project(temp.path(), Duration::days(30)).await;

    let items = generic_list(temp.path(), "issues", Filters::default())
        .await
        .unwrap();
    assert!(
        items.iter().any(|i| i.id == item.id),
        "Active item should not have been affected by cleanup"
    );
}

#[tokio::test]
async fn test_cleanup_no_op_for_empty_project() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;
    write_issue_type_config(temp.path()).await;

    // Should complete without errors when there are no items at all
    run_cleanup_for_project(temp.path(), Duration::days(30)).await;
}
