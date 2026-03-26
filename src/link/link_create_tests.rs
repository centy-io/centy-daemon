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
        inverse: "implemented-by".to_string(),
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

    assert_eq!(result.created_link.kind, "implements");
    assert_eq!(result.inverse_link.kind, "implemented-by");
}

#[tokio::test]
async fn test_create_link_with_custom_inverse_type() {
    // Use the inverse name of a custom type as the link_type
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
        inverse: "implemented-by".to_string(),
        description: None,
    }];

    // Use "implemented-by" as the link type (the inverse)
    let options = CreateLinkOptions {
        source_id: issue1.id.clone(),
        source_type: TargetType::issue(),
        target_id: issue2.id.clone(),
        target_type: TargetType::issue(),
        link_type: "implemented-by".to_string(),
    };

    let result = create_link(temp.path(), options, &custom_types)
        .await
        .unwrap();

    assert_eq!(result.created_link.kind, "implemented-by");
    assert_eq!(result.inverse_link.kind, "implements");
}
