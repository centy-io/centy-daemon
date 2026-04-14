#![allow(clippy::indexing_slicing)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use centy_daemon::config::item_type_config::{
    default_doc_config, write_item_type_config, ItemTypeConfig, ItemTypeFeatures,
};
use centy_daemon::item::entities::issue::{create_issue, CreateIssueOptions};
use centy_daemon::item::generic::storage::generic_create;
use centy_daemon::link::{
    create_link, delete_link, get_available_link_types, list_links, CreateLinkOptions,
    CustomLinkTypeDefinition, DeleteLinkOptions, LinkDirection, LinkError, TargetType,
};
use common::{create_test_dir, init_centy_project};
use mdstore::{CreateOptions, IdStrategy, TypeConfig};

/// Create a slug-based "story" item type config (mirrors real `.centy/stories/config.yaml`).
fn story_item_type_config() -> ItemTypeConfig {
    ItemTypeConfig {
        name: "story".to_string(),
        icon: None,
        identifier: IdStrategy::Slug,
        features: ItemTypeFeatures {
            display_number: false,
            priority: false,
            soft_delete: false,
            assets: false,
            org_sync: false,
            move_item: false,
            duplicate: false,
        },
        statuses: vec!["draft".to_string(), "ready".to_string(), "done".to_string()],
        priority_levels: None,
        custom_fields: vec![],
        template: None,
        listed: true,
    }
}

/// Register the story type and return its `TypeConfig` for use with mdstore.
async fn setup_story_type(project_path: &std::path::Path) -> TypeConfig {
    let config = story_item_type_config();
    write_item_type_config(project_path, "stories", &config)
        .await
        .expect("Should write stories config");
    TypeConfig::from(&config)
}

#[tokio::test]
async fn test_create_link_between_issues() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue1 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue 1".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue 1");

    let issue2 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue 2".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue 2");

    let options = CreateLinkOptions {
        source_id: issue1.id.clone(),
        source_type: TargetType::issue(),
        target_id: issue2.id.clone(),
        target_type: TargetType::issue(),
        link_type: "blocks".to_string(),
    };

    let record = create_link(project_path, options, &[])
        .await
        .expect("Should create link");

    // Source view
    let source_view = record.source_view();
    assert_eq!(source_view.target_id, issue2.id);
    assert_eq!(source_view.link_type, "blocks");
    assert_eq!(source_view.direction, LinkDirection::Source);

    // Target view (from issue2's perspective)
    let target_view = record.target_view();
    assert_eq!(target_view.target_id, issue1.id);
    assert_eq!(target_view.link_type, "blocks");
    assert_eq!(target_view.direction, LinkDirection::Target);
}

#[tokio::test]
async fn test_list_links_shows_both_sides() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue1 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Parent".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let issue2 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Child".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    create_link(
        project_path,
        CreateLinkOptions {
            source_id: issue1.id.clone(),
            source_type: TargetType::issue(),
            target_id: issue2.id.clone(),
            target_type: TargetType::issue(),
            link_type: "parent-of".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    // Source side: direction=source, link_type="parent-of"
    let source_links = list_links(project_path, &issue1.id, TargetType::issue())
        .await
        .unwrap();
    assert_eq!(source_links.len(), 1);
    assert_eq!(source_links[0].link_type, "parent-of");
    assert_eq!(source_links[0].direction, LinkDirection::Source);
    assert_eq!(source_links[0].target_id, issue2.id);

    // Target side: direction=target, link_type="parent-of" (still from source's perspective)
    let target_links = list_links(project_path, &issue2.id, TargetType::issue())
        .await
        .unwrap();
    assert_eq!(target_links.len(), 1);
    assert_eq!(target_links[0].link_type, "parent-of");
    assert_eq!(target_links[0].direction, LinkDirection::Target);
    assert_eq!(target_links[0].target_id, issue1.id);
}

#[tokio::test]
async fn test_create_link_self_link_error() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Self".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let options = CreateLinkOptions {
        source_id: issue.id.clone(),
        source_type: TargetType::issue(),
        target_id: issue.id.clone(),
        target_type: TargetType::issue(),
        link_type: "blocks".to_string(),
    };

    let result = create_link(project_path, options, &[]).await;
    assert!(matches!(result.unwrap_err(), LinkError::SelfLink));
}

#[tokio::test]
async fn test_create_link_invalid_type() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue1 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue 1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let issue2 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue 2".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let options = CreateLinkOptions {
        source_id: issue1.id,
        source_type: TargetType::issue(),
        target_id: issue2.id,
        target_type: TargetType::issue(),
        link_type: "invalid-link-type".to_string(),
    };

    let result = create_link(project_path, options, &[]).await;
    assert!(matches!(result.unwrap_err(), LinkError::InvalidLinkType(_)));
}

#[tokio::test]
async fn test_create_link_source_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Target".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let options = CreateLinkOptions {
        source_id: "nonexistent-uuid".to_string(),
        source_type: TargetType::issue(),
        target_id: issue.id,
        target_type: TargetType::issue(),
        link_type: "blocks".to_string(),
    };

    let result = create_link(project_path, options, &[]).await;
    assert!(matches!(
        result.unwrap_err(),
        LinkError::SourceNotFound(_, _)
    ));
}

#[tokio::test]
async fn test_create_link_target_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Source".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let options = CreateLinkOptions {
        source_id: issue.id,
        source_type: TargetType::issue(),
        target_id: "nonexistent-uuid".to_string(),
        target_type: TargetType::issue(),
        link_type: "blocks".to_string(),
    };

    let result = create_link(project_path, options, &[]).await;
    assert!(matches!(
        result.unwrap_err(),
        LinkError::TargetNotFound(_, _)
    ));
}

#[tokio::test]
async fn test_create_link_already_exists() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue1 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue 1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let issue2 = create_issue(
        project_path,
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
        link_type: "blocks".to_string(),
    };

    create_link(project_path, options.clone(), &[])
        .await
        .unwrap();

    let result = create_link(project_path, options, &[]).await;
    assert!(matches!(result.unwrap_err(), LinkError::LinkAlreadyExists));
}

#[tokio::test]
async fn test_delete_link_by_id() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue1 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue 1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let issue2 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue 2".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let record = create_link(
        project_path,
        CreateLinkOptions {
            source_id: issue1.id.clone(),
            source_type: TargetType::issue(),
            target_id: issue2.id.clone(),
            target_type: TargetType::issue(),
            link_type: "blocks".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    let result = delete_link(
        project_path,
        DeleteLinkOptions {
            link_id: record.id.clone(),
        },
    )
    .await
    .expect("Should delete link");

    assert_eq!(result.deleted_count, 1);

    // Verify link is gone from both sides
    let source_links = list_links(project_path, &issue1.id, TargetType::issue())
        .await
        .unwrap();
    assert!(source_links.is_empty());

    let target_links = list_links(project_path, &issue2.id, TargetType::issue())
        .await
        .unwrap();
    assert!(target_links.is_empty());
}

#[tokio::test]
async fn test_delete_link_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let result = delete_link(
        project_path,
        DeleteLinkOptions {
            link_id: "nonexistent-link-uuid".to_string(),
        },
    )
    .await;
    assert!(matches!(result.unwrap_err(), LinkError::LinkNotFound));
}

#[tokio::test]
async fn test_list_links_empty() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "No Links".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let links = list_links(project_path, &issue.id, TargetType::issue())
        .await
        .expect("Should list links");

    assert!(links.is_empty());
}

#[tokio::test]
async fn test_list_links_multiple() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let main_issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Main Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let related1 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Related 1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let related2 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Related 2".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    create_link(
        project_path,
        CreateLinkOptions {
            source_id: main_issue.id.clone(),
            source_type: TargetType::issue(),
            target_id: related1.id.clone(),
            target_type: TargetType::issue(),
            link_type: "blocks".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    create_link(
        project_path,
        CreateLinkOptions {
            source_id: main_issue.id.clone(),
            source_type: TargetType::issue(),
            target_id: related2.id.clone(),
            target_type: TargetType::issue(),
            link_type: "parent-of".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    let links = list_links(project_path, &main_issue.id, TargetType::issue())
        .await
        .expect("Should list links");

    assert_eq!(links.len(), 2);
}

#[tokio::test]
async fn test_list_links_nonexistent_entity_returns_empty() {
    // With the new model, list_links just scans all link files and filters.
    // A nonexistent entity simply returns an empty list.
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let links = list_links(project_path, "nonexistent-uuid", TargetType::issue())
        .await
        .expect("Should return Ok with empty list for nonexistent entity");

    assert!(links.is_empty());
}

#[tokio::test]
async fn test_get_available_link_types_builtin() {
    let types = get_available_link_types(&[]);

    // All 8 builtin link type names
    assert_eq!(types.len(), 8);
    assert!(types.iter().all(|t| t.is_builtin));

    let names: Vec<&str> = types.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"blocks"));
    assert!(names.contains(&"blocked-by"));
    assert!(names.contains(&"parent-of"));
    assert!(names.contains(&"child-of"));
    assert!(names.contains(&"relates-to"));
    assert!(names.contains(&"related-from"));
    assert!(names.contains(&"duplicates"));
    assert!(names.contains(&"duplicated-by"));
}

#[tokio::test]
async fn test_get_available_link_types_with_custom() {
    let custom = vec![CustomLinkTypeDefinition {
        name: "depends-on".to_string(),
        description: Some("Dependency relationship".to_string()),
    }];

    let types = get_available_link_types(&custom);

    assert_eq!(types.len(), 9); // 8 builtin + 1 custom

    let custom_type = types.iter().find(|t| !t.is_builtin).unwrap();
    assert_eq!(custom_type.name, "depends-on");
}

#[tokio::test]
async fn test_create_link_with_custom_type() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issue1 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue 1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let issue2 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue 2".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let custom = vec![CustomLinkTypeDefinition {
        name: "depends-on".to_string(),
        description: None,
    }];

    let options = CreateLinkOptions {
        source_id: issue1.id.clone(),
        source_type: TargetType::issue(),
        target_id: issue2.id.clone(),
        target_type: TargetType::issue(),
        link_type: "depends-on".to_string(),
    };

    let record = create_link(project_path, options, &custom)
        .await
        .expect("Should create link with custom type");

    assert_eq!(record.link_type, "depends-on");
    assert_eq!(record.source_view().link_type, "depends-on");
    // Target view also shows the stored link_type (source perspective)
    assert_eq!(record.target_view().link_type, "depends-on");
    assert_eq!(record.target_view().direction, LinkDirection::Target);
}

#[tokio::test]
async fn test_all_builtin_link_types_are_valid() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let builtin_types = [
        "blocks",
        "blocked-by",
        "parent-of",
        "child-of",
        "relates-to",
        "related-from",
        "duplicates",
        "duplicated-by",
    ];

    for (i, link_type) in builtin_types.iter().enumerate() {
        let issue1 = create_issue(
            project_path,
            CreateIssueOptions {
                title: format!("Issue A {i}"),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let issue2 = create_issue(
            project_path,
            CreateIssueOptions {
                title: format!("Issue B {i}"),
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
            link_type: (*link_type).to_string(),
        };

        let record = create_link(project_path, options, &[])
            .await
            .unwrap_or_else(|_| panic!("Should create {link_type} link"));

        assert_eq!(record.link_type, *link_type);
        assert_eq!(record.source_view().direction, LinkDirection::Source);
        assert_eq!(record.target_view().direction, LinkDirection::Target);
    }
}

#[tokio::test]
async fn test_target_type_variants() {
    assert_eq!(TargetType::issue().as_str(), "issue");
    assert_eq!(TargetType::new("doc").as_str(), "doc");

    assert_eq!(TargetType::issue().folder_name(), "issues");
    assert_eq!(TargetType::new("doc").folder_name(), "docs");
}

#[tokio::test]
async fn test_target_type_from_str() {
    use std::str::FromStr as _;

    assert_eq!(TargetType::from_str("issue").unwrap(), TargetType::issue());
    assert_eq!(TargetType::from_str("doc").unwrap(), TargetType::new("doc"));
    assert_eq!(TargetType::from_str("ISSUE").unwrap(), TargetType::issue());
    assert_eq!(TargetType::from_str("pr").unwrap().as_str(), "pr");
    assert_eq!(TargetType::from_str("invalid").unwrap().as_str(), "invalid");
}

// Regression tests for issue #361
#[tokio::test]
async fn test_create_link_cross_type_issue_to_doc() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Source Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    let doc_config = TypeConfig::from(&default_doc_config());
    let doc = generic_create(
        project_path,
        "docs",
        &doc_config,
        CreateOptions {
            title: "Target Doc".to_string(),
            body: String::new(),
            id: None,
            status: None,
            priority: None,
            tags: None,
            custom_fields: std::collections::HashMap::new(),
            comment: None,
        },
    )
    .await
    .expect("Should create doc");

    let options = CreateLinkOptions {
        source_id: issue.id.clone(),
        source_type: TargetType::issue(),
        target_id: doc.id.clone(),
        target_type: TargetType::new("doc"),
        link_type: "relates-to".to_string(),
    };

    let record = create_link(project_path, options, &[])
        .await
        .expect("Cross-type link (issue->doc) should succeed");

    assert_eq!(record.source_view().target_id, doc.id);
    assert_eq!(record.source_view().target_type, TargetType::new("doc"));
    assert_eq!(record.source_view().link_type, "relates-to");
    assert_eq!(record.target_view().target_id, issue.id);
    assert_eq!(record.target_view().target_type, TargetType::issue());
    assert_eq!(record.target_view().direction, LinkDirection::Target);
}

#[tokio::test]
async fn test_cross_type_target_not_found_uses_target_type() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Source".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let options = CreateLinkOptions {
        source_id: issue.id.clone(),
        source_type: TargetType::issue(),
        target_id: "nonexistent-doc-uuid".to_string(),
        target_type: TargetType::new("doc"),
        link_type: "relates-to".to_string(),
    };

    let err = create_link(project_path, options, &[]).await.unwrap_err();

    match err {
        LinkError::TargetNotFound(id, ty) => {
            assert_eq!(id, "nonexistent-doc-uuid");
            assert_eq!(ty, TargetType::new("doc"));
        }
        LinkError::IoError(_)
        | LinkError::StoreError(_)
        | LinkError::InvalidLinkType(_)
        | LinkError::SourceNotFound(_, _)
        | LinkError::LinkAlreadyExists
        | LinkError::LinkNotFound
        | LinkError::SelfLink => panic!("Expected TargetNotFound, got {err:?}"),
    }
}

#[tokio::test]
async fn test_list_links_cross_type_doc_entity() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let doc_config = TypeConfig::from(&default_doc_config());
    let doc = generic_create(
        project_path,
        "docs",
        &doc_config,
        CreateOptions {
            title: "Doc".to_string(),
            body: String::new(),
            id: None,
            status: None,
            priority: None,
            tags: None,
            custom_fields: std::collections::HashMap::new(),
            comment: None,
        },
    )
    .await
    .unwrap();

    create_link(
        project_path,
        CreateLinkOptions {
            source_id: issue.id.clone(),
            source_type: TargetType::issue(),
            target_id: doc.id.clone(),
            target_type: TargetType::new("doc"),
            link_type: "relates-to".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    // List links for the doc — should appear with direction=target.
    let doc_links = list_links(project_path, &doc.id, TargetType::new("doc"))
        .await
        .expect("list_links for doc entity should succeed");

    assert_eq!(doc_links.len(), 1);
    // link_type is always stored from source's perspective
    assert_eq!(doc_links[0].link_type, "relates-to");
    assert_eq!(doc_links[0].direction, LinkDirection::Target);
    assert_eq!(doc_links[0].target_id, issue.id);
    assert_eq!(doc_links[0].target_type, TargetType::issue());
}

#[tokio::test]
async fn test_link_record_has_id_for_deletion() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let issue1 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "A".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let issue2 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "B".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let record = create_link(
        project_path,
        CreateLinkOptions {
            source_id: issue1.id.clone(),
            source_type: TargetType::issue(),
            target_id: issue2.id.clone(),
            target_type: TargetType::issue(),
            link_type: "blocks".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    // The record has a UUID that can be used for deletion
    assert!(!record.id.is_empty());

    // The same id appears in the list result
    let links = list_links(project_path, &issue1.id, TargetType::issue())
        .await
        .unwrap();
    assert_eq!(links[0].id, record.id);
}

// ─── Regression tests for issue #417: slug-based item type linking ──────────

/// A slug-based type whose folder name is NOT simply `name + "s"` (e.g. "story"
/// → "stories") must still be linkable.  The naive `folder_name()` helper would
/// produce the wrong folder name, pointing to a non-existent directory and causing a false
/// `LINK_SOURCE_NOT_FOUND` / `LINK_TARGET_NOT_FOUND` error.
#[tokio::test]
async fn test_create_link_with_slug_based_story_type() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let story_config = setup_story_type(project_path).await;

    // Create the story (slug-based identifier → id = slugified title)
    let story = generic_create(
        project_path,
        "stories",
        &story_config,
        CreateOptions {
            title: "My Test Story".to_string(),
            body: String::new(),
            id: None,
            status: Some("draft".to_string()),
            priority: None,
            tags: None,
            custom_fields: std::collections::HashMap::new(),
            comment: None,
        },
    )
    .await
    .expect("Should create story");

    // The slug-based id should be the slugified title
    assert_eq!(story.id, "my-test-story");

    // Create an issue to link the story to
    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Parent Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    // Link story → issue: story is the source with a slug id
    let record = create_link(
        project_path,
        CreateLinkOptions {
            source_id: story.id.clone(),
            source_type: TargetType::new("story"),
            target_id: issue.id.clone(),
            target_type: TargetType::issue(),
            link_type: "child-of".to_string(),
        },
        &[],
    )
    .await
    .expect("Link from slug-based story to issue should succeed");

    assert_eq!(record.source_view().target_id, issue.id);
    assert_eq!(record.source_view().link_type, "child-of");

    // Link issue → story: story is the target with a slug id
    let issue2 = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Second Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let record2 = create_link(
        project_path,
        CreateLinkOptions {
            source_id: issue2.id.clone(),
            source_type: TargetType::issue(),
            target_id: story.id.clone(),
            target_type: TargetType::new("story"),
            link_type: "parent-of".to_string(),
        },
        &[],
    )
    .await
    .expect("Link from issue to slug-based story should succeed");

    assert_eq!(record2.source_view().target_id, story.id);
}

/// When the entity type is not in the item-type registry, `entity_exists`
/// falls back to the naive `folder_name()` helper (appends "s").  The link
/// creation still terminates with the expected `TargetNotFound` error rather
/// than a panic.  This exercises the fallback branch in `resolve_folder`.
#[tokio::test]
async fn test_entity_exists_fallback_for_unregistered_type() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    // "widget" has no config.yaml in .centy → registry.resolve("widget") → None
    // → fallback to folder_name() → "widgets" (doesn't exist) → TargetNotFound
    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Source".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let err = create_link(
        project_path,
        CreateLinkOptions {
            source_id: issue.id.clone(),
            source_type: TargetType::issue(),
            target_id: "some-widget".to_string(),
            target_type: TargetType::new("widget"),
            link_type: "relates-to".to_string(),
        },
        &[],
    )
    .await
    .unwrap_err();

    assert!(
        matches!(err, LinkError::TargetNotFound(_, _)),
        "Expected TargetNotFound for unregistered type, got {err:?}"
    );
}

/// Linking a slug-based story that does not exist should return
/// `SourceNotFound` / `TargetNotFound` (not a spurious `NotFound` caused by
/// checking the wrong folder).
#[tokio::test]
async fn test_link_nonexistent_slug_story_returns_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;
    setup_story_type(project_path).await;

    let issue = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Source does not exist
    let err = create_link(
        project_path,
        CreateLinkOptions {
            source_id: "nonexistent-story".to_string(),
            source_type: TargetType::new("story"),
            target_id: issue.id.clone(),
            target_type: TargetType::issue(),
            link_type: "child-of".to_string(),
        },
        &[],
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, LinkError::SourceNotFound(_, _)),
        "Expected SourceNotFound, got {err:?}"
    );

    // Target does not exist
    let err = create_link(
        project_path,
        CreateLinkOptions {
            source_id: issue.id.clone(),
            source_type: TargetType::issue(),
            target_id: "nonexistent-story".to_string(),
            target_type: TargetType::new("story"),
            link_type: "parent-of".to_string(),
        },
        &[],
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, LinkError::TargetNotFound(_, _)),
        "Expected TargetNotFound, got {err:?}"
    );
}
