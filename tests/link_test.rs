#![allow(clippy::indexing_slicing)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use centy_daemon::item::entities::issue::{create_issue, CreateIssueOptions};
use centy_daemon::link::{
    create_link, delete_link, get_available_link_types, list_links, CreateLinkOptions,
    CustomLinkTypeDefinition, DeleteLinkOptions, LinkError, TargetType,
};
use common::{create_test_dir, init_centy_project};

#[tokio::test]
async fn test_create_link_between_issues() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create two issues
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

    // Create a link
    let options = CreateLinkOptions {
        source_id: issue1.id.clone(),
        source_type: TargetType::Issue,
        target_id: issue2.id.clone(),
        target_type: TargetType::Issue,
        link_type: "blocks".to_string(),
    };

    let result = create_link(project_path, options, &[])
        .await
        .expect("Should create link");

    assert_eq!(result.created_link.target_id, issue2.id);
    assert_eq!(result.created_link.link_type, "blocks");
    assert_eq!(result.inverse_link.target_id, issue1.id);
    assert_eq!(result.inverse_link.link_type, "blocked-by");
}

#[tokio::test]
async fn test_create_link_inverse_created() {
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

    let options = CreateLinkOptions {
        source_id: issue1.id.clone(),
        source_type: TargetType::Issue,
        target_id: issue2.id.clone(),
        target_type: TargetType::Issue,
        link_type: "parent-of".to_string(),
    };

    create_link(project_path, options, &[]).await.unwrap();

    // Verify inverse link exists on target
    let target_links = list_links(project_path, &issue2.id, TargetType::Issue)
        .await
        .unwrap();

    assert_eq!(target_links.links.len(), 1);
    assert_eq!(target_links.links[0].link_type, "child-of");
    assert_eq!(target_links.links[0].target_id, issue1.id);
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
        source_type: TargetType::Issue,
        target_id: issue.id.clone(),
        target_type: TargetType::Issue,
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
        source_type: TargetType::Issue,
        target_id: issue2.id,
        target_type: TargetType::Issue,
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
        source_type: TargetType::Issue,
        target_id: issue.id,
        target_type: TargetType::Issue,
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
        source_type: TargetType::Issue,
        target_id: "nonexistent-uuid".to_string(),
        target_type: TargetType::Issue,
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
        source_type: TargetType::Issue,
        target_id: issue2.id.clone(),
        target_type: TargetType::Issue,
        link_type: "blocks".to_string(),
    };

    // Create first link
    create_link(project_path, options.clone(), &[])
        .await
        .unwrap();

    // Try to create duplicate
    let result = create_link(project_path, options, &[]).await;
    assert!(matches!(result.unwrap_err(), LinkError::LinkAlreadyExists));
}

#[tokio::test]
async fn test_delete_link_success() {
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

    // Create link
    let create_options = CreateLinkOptions {
        source_id: issue1.id.clone(),
        source_type: TargetType::Issue,
        target_id: issue2.id.clone(),
        target_type: TargetType::Issue,
        link_type: "blocks".to_string(),
    };
    create_link(project_path, create_options, &[])
        .await
        .unwrap();

    // Delete link
    let delete_options = DeleteLinkOptions {
        source_id: issue1.id.clone(),
        source_type: TargetType::Issue,
        target_id: issue2.id.clone(),
        target_type: TargetType::Issue,
        link_type: Some("blocks".to_string()),
    };
    let result = delete_link(project_path, delete_options, &[])
        .await
        .expect("Should delete link");

    assert_eq!(result.deleted_count, 2); // forward + inverse

    // Verify links are gone
    let source_links = list_links(project_path, &issue1.id, TargetType::Issue)
        .await
        .unwrap();
    assert!(source_links.links.is_empty());

    let target_links = list_links(project_path, &issue2.id, TargetType::Issue)
        .await
        .unwrap();
    assert!(target_links.links.is_empty());
}

#[tokio::test]
async fn test_delete_link_not_found() {
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

    let delete_options = DeleteLinkOptions {
        source_id: issue1.id,
        source_type: TargetType::Issue,
        target_id: issue2.id,
        target_type: TargetType::Issue,
        link_type: Some("blocks".to_string()),
    };

    let result = delete_link(project_path, delete_options, &[]).await;
    assert!(matches!(result.unwrap_err(), LinkError::LinkNotFound));
}

#[tokio::test]
async fn test_delete_all_links_between_entities() {
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

    // Create multiple links
    create_link(
        project_path,
        CreateLinkOptions {
            source_id: issue1.id.clone(),
            source_type: TargetType::Issue,
            target_id: issue2.id.clone(),
            target_type: TargetType::Issue,
            link_type: "blocks".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    create_link(
        project_path,
        CreateLinkOptions {
            source_id: issue1.id.clone(),
            source_type: TargetType::Issue,
            target_id: issue2.id.clone(),
            target_type: TargetType::Issue,
            link_type: "relates-to".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    // Delete all links (no link_type specified)
    let delete_options = DeleteLinkOptions {
        source_id: issue1.id.clone(),
        source_type: TargetType::Issue,
        target_id: issue2.id.clone(),
        target_type: TargetType::Issue,
        link_type: None,
    };

    let result = delete_link(project_path, delete_options, &[])
        .await
        .expect("Should delete all links");

    assert!(result.deleted_count >= 2);
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

    let links = list_links(project_path, &issue.id, TargetType::Issue)
        .await
        .expect("Should list links");

    assert!(links.links.is_empty());
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

    // Create links from main to others
    create_link(
        project_path,
        CreateLinkOptions {
            source_id: main_issue.id.clone(),
            source_type: TargetType::Issue,
            target_id: related1.id.clone(),
            target_type: TargetType::Issue,
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
            source_type: TargetType::Issue,
            target_id: related2.id.clone(),
            target_type: TargetType::Issue,
            link_type: "parent-of".to_string(),
        },
        &[],
    )
    .await
    .unwrap();

    let links = list_links(project_path, &main_issue.id, TargetType::Issue)
        .await
        .expect("Should list links");

    assert_eq!(links.links.len(), 2);
}

#[tokio::test]
async fn test_list_links_entity_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let result = list_links(project_path, "nonexistent-uuid", TargetType::Issue).await;
    assert!(matches!(
        result.unwrap_err(),
        LinkError::SourceNotFound(_, _)
    ));
}

#[tokio::test]
async fn test_get_available_link_types_builtin() {
    let types = get_available_link_types(&[]);

    // Should have 4 builtin pairs
    assert_eq!(types.len(), 4);
    assert!(types.iter().all(|t| t.is_builtin));

    let names: Vec<&str> = types.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"blocks"));
    assert!(names.contains(&"parent-of"));
    assert!(names.contains(&"relates-to"));
    assert!(names.contains(&"duplicates"));
}

#[tokio::test]
async fn test_get_available_link_types_with_custom() {
    let custom = vec![CustomLinkTypeDefinition {
        name: "depends-on".to_string(),
        inverse: "dependency-of".to_string(),
        description: Some("Dependency relationship".to_string()),
    }];

    let types = get_available_link_types(&custom);

    assert_eq!(types.len(), 5); // 4 builtin + 1 custom

    let custom_type = types.iter().find(|t| !t.is_builtin).unwrap();
    assert_eq!(custom_type.name, "depends-on");
    assert_eq!(custom_type.inverse, "dependency-of");
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
        inverse: "dependency-of".to_string(),
        description: None,
    }];

    let options = CreateLinkOptions {
        source_id: issue1.id.clone(),
        source_type: TargetType::Issue,
        target_id: issue2.id.clone(),
        target_type: TargetType::Issue,
        link_type: "depends-on".to_string(),
    };

    let result = create_link(project_path, options, &custom)
        .await
        .expect("Should create link with custom type");

    assert_eq!(result.created_link.link_type, "depends-on");
    assert_eq!(result.inverse_link.link_type, "dependency-of");
}

#[tokio::test]
async fn test_all_builtin_link_types() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let builtin_types = ["blocks", "parent-of", "relates-to", "duplicates"];
    let expected_inverses = ["blocked-by", "child-of", "related-from", "duplicated-by"];

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
            source_type: TargetType::Issue,
            target_id: issue2.id.clone(),
            target_type: TargetType::Issue,
            link_type: (*link_type).to_string(),
        };

        let result = create_link(project_path, options, &[])
            .await
            .unwrap_or_else(|_| panic!("Should create {link_type} link"));

        assert_eq!(result.created_link.link_type, *link_type);
        assert_eq!(result.inverse_link.link_type, expected_inverses[i]);
    }
}

#[tokio::test]
async fn test_target_type_variants() {
    assert_eq!(TargetType::Issue.as_str(), "issue");
    assert_eq!(TargetType::Doc.as_str(), "doc");
    assert_eq!(TargetType::Pr.as_str(), "pr");

    assert_eq!(TargetType::Issue.folder_name(), "issues");
    assert_eq!(TargetType::Doc.folder_name(), "docs");
    assert_eq!(TargetType::Pr.folder_name(), "prs");
}

#[tokio::test]
async fn test_target_type_from_str() {
    use std::str::FromStr;

    assert_eq!(TargetType::from_str("issue").unwrap(), TargetType::Issue);
    assert_eq!(TargetType::from_str("doc").unwrap(), TargetType::Doc);
    assert_eq!(TargetType::from_str("pr").unwrap(), TargetType::Pr);
    assert_eq!(TargetType::from_str("ISSUE").unwrap(), TargetType::Issue);
    assert!(TargetType::from_str("invalid").is_err());
}
