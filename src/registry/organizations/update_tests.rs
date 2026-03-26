//! Tests for `update::update_organization` covering missed branches.
//! `super` here is `update`, `super::super` is `organizations`.
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::await_holding_lock)]

use super::super::create_organization;
use super::super::delete_organization;
use super::update_organization;
use std::sync::atomic::{AtomicU32, Ordering};

static UPDATE_COUNTER: AtomicU32 = AtomicU32::new(0);

fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
    super::super::acquire_org_test_lock()
}

fn unique_slug(prefix: &str) -> String {
    let n = UPDATE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    format!("{prefix}-update-{pid}-{n}")
}

/// Covers `handle_slug_rename` when `new_slug` is None (no rename).
#[tokio::test]
async fn test_update_no_slug_rename() {
    let _lock = acquire_lock();
    let slug = unique_slug("no-rename");
    create_organization(Some(&slug), "Original", None)
        .await
        .expect("create");

    let result = update_organization(&slug, Some("Updated Name"), None, None)
        .await
        .expect("update");

    assert_eq!(result.slug, slug, "slug unchanged");
    assert_eq!(result.name, "Updated Name");

    drop(delete_organization(&slug, false).await);
}

/// Covers the `ns.is_empty()` branch in `handle_slug_rename` (`new_slug` is
/// `Some("")` — treated as "no rename").
#[tokio::test]
async fn test_update_empty_new_slug_no_rename() {
    let _lock = acquire_lock();
    let slug = unique_slug("empty-ns");
    create_organization(Some(&slug), "Test", None)
        .await
        .expect("create");

    let result = update_organization(&slug, None, None, Some(""))
        .await
        .expect("update with empty new slug");

    assert_eq!(result.slug, slug, "slug unchanged when new_slug is empty");

    drop(delete_organization(&slug, false).await);
}

/// Covers the `AlreadyExists` error when renaming to an occupied slug.
#[tokio::test]
async fn test_update_slug_rename_conflict() {
    let _lock = acquire_lock();
    let slug_a = unique_slug("conflict-a");
    let slug_b = unique_slug("conflict-b");

    create_organization(Some(&slug_a), "Org A", None)
        .await
        .expect("create a");
    create_organization(Some(&slug_b), "Org B", None)
        .await
        .expect("create b");

    let result = update_organization(&slug_a, None, None, Some(&slug_b)).await;
    assert!(result.is_err(), "should fail: slug_b already exists");

    drop(delete_organization(&slug_a, false).await);
    drop(delete_organization(&slug_b, false).await);
}

/// Covers the `NotFound` error when updating a non-existent slug.
#[tokio::test]
async fn test_update_org_not_found() {
    let _lock = acquire_lock();
    let result = update_organization("ghost-org-update-xyz", Some("X"), None, None).await;
    assert!(result.is_err(), "should error for missing org");
}

/// Covers description clearing (empty string → None) branch.
#[tokio::test]
async fn test_update_description_cleared() {
    let _lock = acquire_lock();
    let slug = unique_slug("desc-clr");
    create_organization(Some(&slug), "Org", Some("initial"))
        .await
        .expect("create");

    let result = update_organization(&slug, None, Some(""), None)
        .await
        .expect("update");

    assert_eq!(result.description, None, "empty string clears description");

    drop(delete_organization(&slug, false).await);
}

/// Covers the slug rename success path.
#[tokio::test]
async fn test_update_slug_rename_success() {
    let _lock = acquire_lock();
    let old_slug = unique_slug("rename-old");
    let new_slug = unique_slug("rename-new");

    create_organization(Some(&old_slug), "Rename Me", None)
        .await
        .expect("create");

    let result = update_organization(&old_slug, None, None, Some(&new_slug))
        .await
        .expect("rename");

    assert_eq!(result.slug, new_slug);
    assert_eq!(result.name, "Rename Me");

    drop(delete_organization(&new_slug, false).await);
}
