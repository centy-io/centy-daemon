//! Additional tests for `link/crud_fns/create.rs` covering missed branches.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::item::entities::issue::{create_issue, CreateIssueOptions};

async fn setup_project(temp: &std::path::Path) {
    use crate::reconciliation::{execute_reconciliation, ReconciliationDecisions};
    let decisions = ReconciliationDecisions::default();
    execute_reconciliation(temp, decisions, true)
        .await
        .expect("Failed to initialize centy project");
}

// ─── create_link with custom types ───────────────────────────────────────────

#[tokio::test]
async fn test_create_link_with_custom_type() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let issue1 = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Issue 1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let issue2 = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Issue 2".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let custom_types = vec![CustomLinkTypeDefinition {
        name: "implements".to_string(),
        description: None,
    }];

    let options = CreateLinkOptions {
        source_id: issue1.id.clone(),
        source_type: TargetType::issue(),
        target_id: issue2.id.clone(),
        target_type: TargetType::issue(),
        link_type: "implements".to_string(),
    };

    let result = create_link(temp.path(), options, &custom_types)
        .await
        .unwrap();

    assert_eq!(result.link_type, "implements");
    assert_eq!(result.source_id, issue1.id);
    assert_eq!(result.target_id, issue2.id);
}

#[tokio::test]
async fn test_create_link_invalid_type_returns_error() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let issue1 = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Issue 1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let issue2 = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Issue 2".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let options = CreateLinkOptions {
        source_id: issue1.id.clone(),
        source_type: TargetType::issue(),
        target_id: issue2.id.clone(),
        target_type: TargetType::issue(),
        link_type: "not-a-valid-type".to_string(),
    };

    let result = create_link(temp.path(), options, &[]).await;
    assert!(matches!(result, Err(LinkError::InvalidLinkType(_))));
}

#[tokio::test]
async fn test_create_link_duplicate_returns_error() {
    let temp = tempfile::tempdir().unwrap();
    setup_project(temp.path()).await;

    let issue1 = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Issue 1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let issue2 = create_issue(
        temp.path(),
        CreateIssueOptions {
            title: "Issue 2".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let make_opts = || CreateLinkOptions {
        source_id: issue1.id.clone(),
        source_type: TargetType::issue(),
        target_id: issue2.id.clone(),
        target_type: TargetType::issue(),
        link_type: "blocks".to_string(),
    };

    create_link(temp.path(), make_opts(), &[]).await.unwrap();
    let result = create_link(temp.path(), make_opts(), &[]).await;
    assert!(matches!(result, Err(LinkError::LinkAlreadyExists)));
}
