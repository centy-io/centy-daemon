//! Tests for `query::list_organizations` and `query::get_organization`.
//! `super` here is `query`, `super::super` is `organizations`.
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::await_holding_lock)]

use super::super::{create_organization, delete_organization};
use super::{get_organization, list_organizations};
use std::sync::atomic::{AtomicU32, Ordering};

static QUERY_COUNTER: AtomicU32 = AtomicU32::new(0);

fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
    super::super::acquire_org_test_lock()
}

fn unique_slug(prefix: &str) -> String {
    let n = QUERY_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    format!("{prefix}-qry-{pid}-{n}")
}

/// Covers the `Ok(None)` branch in `get_organization`.
#[tokio::test]
async fn test_get_organization_missing_returns_none() {
    let _lock = acquire_lock();

    let result = get_organization("definitely-not-there-query-abc-xyz")
        .await
        .expect("should not error");

    assert!(result.is_none(), "expected None for non-existent org");
}

/// Covers the `Some(OrganizationInfo {...})` path in `get_organization`.
#[tokio::test]
async fn test_get_organization_found() {
    let _lock = acquire_lock();

    let slug = unique_slug("found");
    create_organization(Some(&slug), "Query Found", Some("desc"))
        .await
        .expect("create");

    let result = get_organization(&slug)
        .await
        .expect("query ok")
        .expect("should be Some");

    assert_eq!(result.slug, slug);
    assert_eq!(result.name, "Query Found");
    assert_eq!(result.description, Some("desc".to_string()));
    assert_eq!(result.project_count, 0);

    drop(delete_organization(&slug, false).await);
}

/// Covers `list_organizations` returning a sorted vec.
#[tokio::test]
async fn test_list_organizations_sorted_by_name() {
    let _lock = acquire_lock();

    let slug_z = unique_slug("zzz");
    let slug_a = unique_slug("aaa");
    create_organization(Some(&slug_z), "ZZZ Query Org", None)
        .await
        .expect("create z");
    create_organization(Some(&slug_a), "AAA Query Org", None)
        .await
        .expect("create a");

    let orgs = list_organizations().await.expect("list");

    let pos_z = orgs.iter().position(|o| o.slug == slug_z);
    let pos_a = orgs.iter().position(|o| o.slug == slug_a);

    assert!(pos_z.is_some() && pos_a.is_some());
    assert!(
        pos_a.unwrap() < pos_z.unwrap(),
        "list should be sorted by name ascending"
    );

    drop(delete_organization(&slug_z, false).await);
    drop(delete_organization(&slug_a, false).await);
}
