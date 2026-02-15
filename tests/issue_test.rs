#![allow(clippy::indexing_slicing)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use centy_daemon::config::item_type_config::default_issue_config;
use centy_daemon::config::CentyConfig;
use centy_daemon::item::entities::issue::{
    create_issue, delete_issue, get_issue, is_uuid, list_issues, move_issue, update_issue,
    CreateIssueOptions, IssueCrudError, IssueError, MoveIssueOptions, UpdateIssueOptions,
};
use centy_daemon::item::generic::{generic_duplicate, DuplicateGenericItemOptions};
use common::{create_test_dir, init_centy_project};
use std::collections::HashMap;

#[tokio::test]
async fn test_create_issue_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize centy first
    init_centy_project(project_path).await;

    // Create an issue with numeric priority (1 = highest)
    let options = CreateIssueOptions {
        title: "Test Issue".to_string(),
        description: "This is a test issue".to_string(),
        priority: Some(1), // high priority
        status: Some("open".to_string()),
        custom_fields: HashMap::new(),
        ..Default::default()
    };

    let result = create_issue(project_path, options)
        .await
        .expect("Should create issue");

    // Issue ID should be a UUID
    assert!(is_uuid(&result.id), "Issue ID should be a UUID");
    assert_eq!(result.display_number, 1);
    assert_eq!(result.created_files.len(), 1); // {id}.md (new YAML frontmatter format)

    // Verify single .md file exists with new format
    let issue_file = project_path.join(format!(".centy/issues/{}.md", result.id));
    assert!(issue_file.exists(), "Issue file should exist");
}

#[tokio::test]
async fn test_create_issue_increments_display_number() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create first issue
    let options1 = CreateIssueOptions {
        title: "First Issue".to_string(),
        ..Default::default()
    };
    let result1 = create_issue(project_path, options1)
        .await
        .expect("Should create");
    assert!(is_uuid(&result1.id), "Issue ID should be a UUID");
    assert_eq!(result1.display_number, 1);

    // Create second issue
    let options2 = CreateIssueOptions {
        title: "Second Issue".to_string(),
        ..Default::default()
    };
    let result2 = create_issue(project_path, options2)
        .await
        .expect("Should create");
    assert!(is_uuid(&result2.id), "Issue ID should be a UUID");
    assert_eq!(result2.display_number, 2);

    // Create third issue
    let options3 = CreateIssueOptions {
        title: "Third Issue".to_string(),
        ..Default::default()
    };
    let result3 = create_issue(project_path, options3)
        .await
        .expect("Should create");
    assert!(is_uuid(&result3.id), "Issue ID should be a UUID");
    assert_eq!(result3.display_number, 3);

    // All IDs should be unique
    assert_ne!(result1.id, result2.id);
    assert_ne!(result2.id, result3.id);
}

#[tokio::test]
async fn test_create_issue_requires_init() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Don't initialize - try to create issue
    let options = CreateIssueOptions {
        title: "Test Issue".to_string(),
        ..Default::default()
    };

    let result = create_issue(project_path, options).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IssueError::NotInitialized));
}

#[tokio::test]
async fn test_create_issue_requires_title() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Try to create issue without title
    let options = CreateIssueOptions {
        title: String::new(),
        ..Default::default()
    };

    let result = create_issue(project_path, options).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IssueError::TitleRequired));
}

#[tokio::test]
async fn test_create_issue_default_priority_and_status() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue without specifying priority/status
    let options = CreateIssueOptions {
        title: "Test Issue".to_string(),
        ..Default::default()
    };

    let result = create_issue(project_path, options)
        .await
        .expect("Should create");

    // Get the issue and verify defaults
    // Default priority with 3 levels (high/medium/low) is 2 (medium)
    let issue = get_issue(project_path, &result.id)
        .await
        .expect("Should get issue");
    assert_eq!(issue.metadata.priority, 2); // medium
    assert_eq!(issue.metadata.status, "open");
}

#[tokio::test]
async fn test_get_issue_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create an issue with numeric priority
    let options = CreateIssueOptions {
        title: "My Test Issue".to_string(),
        description: "Description here".to_string(),
        priority: Some(1), // high
        status: Some("in-progress".to_string()),
        custom_fields: HashMap::new(),
        ..Default::default()
    };
    let result = create_issue(project_path, options)
        .await
        .expect("Should create");

    // Get the issue by UUID
    let issue = get_issue(project_path, &result.id)
        .await
        .expect("Should get issue");

    assert!(is_uuid(&issue.id), "Issue ID should be a UUID");
    assert_eq!(issue.metadata.display_number, 1);
    assert_eq!(issue.title, "My Test Issue");
    assert_eq!(issue.description, "Description here");
    assert_eq!(issue.metadata.priority, 1); // high
    assert_eq!(issue.metadata.status, "in-progress");
}

#[tokio::test]
async fn test_get_issue_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let result = get_issue(project_path, "9999").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        IssueCrudError::IssueNotFound(_)
    ));
}

#[tokio::test]
async fn test_list_issues_empty() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issues = list_issues(project_path, None, None, None, false)
        .await
        .expect("Should list issues");

    assert!(issues.is_empty());
}

#[tokio::test]
async fn test_list_issues_returns_all() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create multiple issues
    for i in 1..=3 {
        let options = CreateIssueOptions {
            title: format!("Issue {i}"),
            ..Default::default()
        };
        create_issue(project_path, options)
            .await
            .expect("Should create");
    }

    let issues = list_issues(project_path, None, None, None, false)
        .await
        .expect("Should list issues");

    assert_eq!(issues.len(), 3);
    // Should be sorted by display number
    assert_eq!(issues[0].metadata.display_number, 1);
    assert_eq!(issues[1].metadata.display_number, 2);
    assert_eq!(issues[2].metadata.display_number, 3);
    // All should have UUID IDs
    for issue in &issues {
        assert!(is_uuid(&issue.id), "Issue ID should be a UUID");
    }
}

#[tokio::test]
async fn test_list_issues_filter_by_status() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issues with different statuses
    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Open Issue".to_string(),
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Closed Issue".to_string(),
            status: Some("closed".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Filter by status
    let open_issues = list_issues(project_path, Some("open"), None, None, false)
        .await
        .expect("Should list");
    assert_eq!(open_issues.len(), 1);
    assert_eq!(open_issues[0].title, "Open Issue");

    let closed_issues = list_issues(project_path, Some("closed"), None, None, false)
        .await
        .expect("Should list");
    assert_eq!(closed_issues.len(), 1);
    assert_eq!(closed_issues[0].title, "Closed Issue");
}

#[tokio::test]
async fn test_list_issues_filter_by_priority() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issues with different priorities (numeric)
    create_issue(
        project_path,
        CreateIssueOptions {
            title: "High Priority".to_string(),
            priority: Some(1), // high
            ..Default::default()
        },
    )
    .await
    .unwrap();

    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Low Priority".to_string(),
            priority: Some(3), // low
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Filter by priority (numeric)
    let high_issues = list_issues(project_path, None, Some(1), None, false)
        .await
        .expect("Should list");
    assert_eq!(high_issues.len(), 1);
    assert_eq!(high_issues[0].title, "High Priority");
}

#[tokio::test]
async fn test_update_issue_title() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue
    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Original Title".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Update title
    let options = UpdateIssueOptions {
        title: Some("Updated Title".to_string()),
        ..Default::default()
    };

    let result = update_issue(project_path, &created.id, options)
        .await
        .expect("Should update");

    assert_eq!(result.issue.title, "Updated Title");

    // Verify persisted
    let issue = get_issue(project_path, &created.id).await.unwrap();
    assert_eq!(issue.title, "Updated Title");
}

#[tokio::test]
async fn test_update_issue_status() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test".to_string(),
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Update status
    let result = update_issue(
        project_path,
        &created.id,
        UpdateIssueOptions {
            status: Some("closed".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("Should update");

    assert_eq!(result.issue.metadata.status, "closed");
}

#[tokio::test]
async fn test_update_issue_preserves_unchanged_fields() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create with specific values
    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Original".to_string(),
            description: "Original description".to_string(),
            priority: Some(1), // high
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Only update title
    let result = update_issue(
        project_path,
        &created.id,
        UpdateIssueOptions {
            title: Some("New Title".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Other fields should be preserved
    assert_eq!(result.issue.title, "New Title");
    assert_eq!(result.issue.description, "Original description");
    assert_eq!(result.issue.metadata.priority, 1); // high
    assert_eq!(result.issue.metadata.status, "open");
}

#[tokio::test]
async fn test_update_issue_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let result = update_issue(
        project_path,
        "9999",
        UpdateIssueOptions {
            title: Some("New".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        IssueCrudError::IssueNotFound(_)
    ));
}

#[tokio::test]
async fn test_delete_issue_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue
    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "To Delete".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Verify issue file exists (new format: {id}.md)
    let issue_file = project_path.join(format!(".centy/issues/{}.md", created.id));
    assert!(issue_file.exists());

    // Delete it
    delete_issue(project_path, &created.id)
        .await
        .expect("Should delete");

    // Verify issue file is gone
    assert!(!issue_file.exists());

    // Verify not in list
    let issues = list_issues(project_path, None, None, None, false)
        .await
        .unwrap();
    assert!(issues.is_empty());
}

#[tokio::test]
async fn test_delete_issue_removes_files() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue
    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Verify issue file exists (new format: {id}.md)
    let issue_file = project_path
        .join(".centy")
        .join("issues")
        .join(format!("{}.md", &created.id));
    assert!(
        issue_file.exists(),
        "Issue file should exist after creation"
    );

    // Delete issue
    let _result = delete_issue(project_path, &created.id).await.unwrap();

    // Verify issue file is removed
    assert!(
        !issue_file.exists(),
        "Issue file should be removed after deletion"
    );
}

#[tokio::test]
async fn test_delete_issue_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let result = delete_issue(project_path, "9999").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        IssueCrudError::IssueNotFound(_)
    ));
}

#[tokio::test]
async fn test_issue_with_custom_fields() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue with custom fields
    let mut custom_fields = HashMap::new();
    custom_fields.insert("assignee".to_string(), "alice".to_string());
    custom_fields.insert("component".to_string(), "backend".to_string());

    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Custom Fields Test".to_string(),
            custom_fields,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Get and verify
    let issue = get_issue(project_path, &created.id).await.unwrap();
    assert_eq!(
        issue.metadata.custom_fields.get("assignee"),
        Some(&"alice".to_string())
    );
    assert_eq!(
        issue.metadata.custom_fields.get("component"),
        Some(&"backend".to_string())
    );
}

#[tokio::test]
async fn test_update_issue_custom_fields() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create with initial custom fields
    let mut initial_fields = HashMap::new();
    initial_fields.insert("assignee".to_string(), "alice".to_string());

    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test".to_string(),
            custom_fields: initial_fields,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Update custom fields
    let mut new_fields = HashMap::new();
    new_fields.insert("assignee".to_string(), "bob".to_string());
    new_fields.insert("reviewer".to_string(), "charlie".to_string());

    let result = update_issue(
        project_path,
        &created.id,
        UpdateIssueOptions {
            custom_fields: new_fields,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(
        result.issue.metadata.custom_fields.get("assignee"),
        Some(&"bob".to_string())
    );
    assert_eq!(
        result.issue.metadata.custom_fields.get("reviewer"),
        Some(&"charlie".to_string())
    );
}

#[tokio::test]
async fn test_create_issue_validates_priority_range() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Try to create issue with out-of-range priority (default is 3 levels)
    let options = CreateIssueOptions {
        title: "Invalid Priority".to_string(),
        priority: Some(5), // Invalid - max is 3
        ..Default::default()
    };

    let result = create_issue(project_path, options).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        IssueError::InvalidPriority(_)
    ));
}

#[tokio::test]
async fn test_update_issue_priority() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create with low priority
    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test".to_string(),
            priority: Some(3), // low
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Update to high priority
    let result = update_issue(
        project_path,
        &created.id,
        UpdateIssueOptions {
            priority: Some(1), // high
            ..Default::default()
        },
    )
    .await
    .expect("Should update");

    assert_eq!(result.issue.metadata.priority, 1);
}

// ============ Move Issue Tests ============

#[tokio::test]
async fn test_move_issue_success() {
    let source_dir = create_test_dir();
    let target_dir = create_test_dir();
    let source_path = source_dir.path();
    let target_path = target_dir.path();

    init_centy_project(source_path).await;
    init_centy_project(target_path).await;

    // Create issue in source
    let created = create_issue(
        source_path,
        CreateIssueOptions {
            title: "Issue to Move".to_string(),
            description: "Description".to_string(),
            priority: Some(1),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Move to target
    let result = move_issue(MoveIssueOptions {
        source_project_path: source_path.to_path_buf(),
        target_project_path: target_path.to_path_buf(),
        issue_id: created.id.clone(),
    })
    .await
    .expect("Should move issue");

    // Verify issue exists in target with same UUID
    assert_eq!(result.issue.id, created.id);
    assert_eq!(result.issue.metadata.display_number, 1);
    assert_eq!(result.issue.title, "Issue to Move");

    // Verify issue no longer exists in source
    let source_result = get_issue(source_path, &created.id).await;
    assert!(source_result.is_err());

    // Verify exists in target
    let target_issue = get_issue(target_path, &created.id).await;
    assert!(target_issue.is_ok());
}

#[tokio::test]
async fn test_move_issue_preserves_uuid() {
    let source_dir = create_test_dir();
    let target_dir = create_test_dir();
    let source_path = source_dir.path();
    let target_path = target_dir.path();

    init_centy_project(source_path).await;
    init_centy_project(target_path).await;

    let created = create_issue(
        source_path,
        CreateIssueOptions {
            title: "Linked Issue".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let original_uuid = created.id.clone();

    let result = move_issue(MoveIssueOptions {
        source_project_path: source_path.to_path_buf(),
        target_project_path: target_path.to_path_buf(),
        issue_id: created.id,
    })
    .await
    .unwrap();

    // UUID must be preserved
    assert_eq!(result.issue.id, original_uuid);
}

#[tokio::test]
async fn test_move_issue_assigns_new_display_number() {
    let source_dir = create_test_dir();
    let target_dir = create_test_dir();
    let source_path = source_dir.path();
    let target_path = target_dir.path();

    init_centy_project(source_path).await;
    init_centy_project(target_path).await;

    // Create existing issues in target
    create_issue(
        target_path,
        CreateIssueOptions {
            title: "Existing 1".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    create_issue(
        target_path,
        CreateIssueOptions {
            title: "Existing 2".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Create and move issue from source
    let created = create_issue(
        source_path,
        CreateIssueOptions {
            title: "To Move".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(created.display_number, 1); // First in source

    let result = move_issue(MoveIssueOptions {
        source_project_path: source_path.to_path_buf(),
        target_project_path: target_path.to_path_buf(),
        issue_id: created.id,
    })
    .await
    .unwrap();

    // Should be 3 in target (after 1 and 2)
    assert_eq!(result.issue.metadata.display_number, 3);
}

#[tokio::test]
async fn test_move_issue_same_project_fails() {
    let dir = create_test_dir();
    let project_path = dir.path();

    init_centy_project(project_path).await;

    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let result = move_issue(MoveIssueOptions {
        source_project_path: project_path.to_path_buf(),
        target_project_path: project_path.to_path_buf(),
        issue_id: created.id,
    })
    .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IssueCrudError::SameProject));
}

// ============ Duplicate Issue Tests ============

#[tokio::test]
async fn test_duplicate_issue_same_project() {
    let dir = create_test_dir();
    let project_path = dir.path();

    init_centy_project(project_path).await;

    let original = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Original Issue".to_string(),
            description: "Original description".to_string(),
            priority: Some(1),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let config = CentyConfig::default();
    let item_type_config = default_issue_config(&config);

    let result = generic_duplicate(
        &item_type_config,
        DuplicateGenericItemOptions {
            source_project_path: project_path.to_path_buf(),
            target_project_path: project_path.to_path_buf(),
            item_id: original.id.clone(),
            new_id: None,
            new_title: None,
        },
    )
    .await
    .expect("Should duplicate issue");

    // New UUID
    assert_ne!(result.item.id, original.id);
    // New display number
    assert_eq!(result.item.frontmatter.display_number, Some(2));
    // Default title
    assert_eq!(result.item.title, "Copy of Original Issue");
    // Same description
    assert_eq!(result.item.body, "Original description");
    // Same priority
    assert_eq!(result.item.frontmatter.priority, Some(1));
}

#[tokio::test]
async fn test_duplicate_issue_with_custom_title() {
    let dir = create_test_dir();
    let project_path = dir.path();

    init_centy_project(project_path).await;

    let original = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Original".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let config = CentyConfig::default();
    let item_type_config = default_issue_config(&config);

    let result = generic_duplicate(
        &item_type_config,
        DuplicateGenericItemOptions {
            source_project_path: project_path.to_path_buf(),
            target_project_path: project_path.to_path_buf(),
            item_id: original.id,
            new_id: None,
            new_title: Some("Custom Title".to_string()),
        },
    )
    .await
    .unwrap();

    assert_eq!(result.item.title, "Custom Title");
}

#[tokio::test]
async fn test_duplicate_issue_to_different_project() {
    let source_dir = create_test_dir();
    let target_dir = create_test_dir();
    let source_path = source_dir.path();
    let target_path = target_dir.path();

    init_centy_project(source_path).await;
    init_centy_project(target_path).await;

    let original = create_issue(
        source_path,
        CreateIssueOptions {
            title: "Original".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let config = CentyConfig::default();
    let item_type_config = default_issue_config(&config);

    let result = generic_duplicate(
        &item_type_config,
        DuplicateGenericItemOptions {
            source_project_path: source_path.to_path_buf(),
            target_project_path: target_path.to_path_buf(),
            item_id: original.id.clone(),
            new_id: None,
            new_title: None,
        },
    )
    .await
    .unwrap();

    // Original still exists in source
    let original_still_exists = get_issue(source_path, &original.id).await;
    assert!(original_still_exists.is_ok());

    // Duplicate exists in target
    let duplicate_exists = get_issue(target_path, &result.item.id).await;
    assert!(duplicate_exists.is_ok());

    // Different UUIDs
    assert_ne!(result.item.id, original.id);
}

// ============ Planning State Tests ============

#[tokio::test]
async fn test_create_issue_with_planning_status_adds_note() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue with planning status
    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Planning Issue".to_string(),
            description: "Need to plan this".to_string(),
            status: Some("planning".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    // Read issue file directly to verify planning note (new format: {id}.md)
    let issue_file_path = project_path
        .join(".centy/issues")
        .join(format!("{}.md", &created.id));
    let content = tokio::fs::read_to_string(&issue_file_path).await.unwrap();

    assert!(
        content.contains("> **Planning Mode**"),
        "Should contain planning note"
    );
    assert!(content.contains("# Planning Issue"), "Should contain title");
}

#[tokio::test]
async fn test_create_issue_without_planning_status_no_note() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue with open status
    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Regular Issue".to_string(),
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("Should create issue");

    // Read issue file directly (new format: {id}.md)
    let issue_file_path = project_path
        .join(".centy/issues")
        .join(format!("{}.md", &created.id));
    let content = tokio::fs::read_to_string(&issue_file_path).await.unwrap();

    assert!(
        !content.contains("> **Planning Mode**"),
        "Should NOT contain planning note"
    );
    // With YAML frontmatter, content starts with --- not # Title
    assert!(content.contains("# Regular Issue"), "Should contain title");
}

#[tokio::test]
async fn test_update_issue_to_planning_adds_note() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue with open status
    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Verify no planning note initially (new format: {id}.md)
    let issue_file_path = project_path
        .join(".centy/issues")
        .join(format!("{}.md", &created.id));
    let initial_content = tokio::fs::read_to_string(&issue_file_path).await.unwrap();
    assert!(!initial_content.contains("> **Planning Mode**"));

    // Update to planning status
    update_issue(
        project_path,
        &created.id,
        UpdateIssueOptions {
            status: Some("planning".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("Should update issue");

    // Verify planning note was added
    let updated_content = tokio::fs::read_to_string(&issue_file_path).await.unwrap();
    assert!(
        updated_content.contains("> **Planning Mode**"),
        "Should add planning note"
    );
}

#[tokio::test]
async fn test_update_issue_from_planning_removes_note() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue with planning status
    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test Issue".to_string(),
            status: Some("planning".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Verify planning note exists (new format: {id}.md)
    let issue_file_path = project_path
        .join(".centy/issues")
        .join(format!("{}.md", &created.id));
    let initial_content = tokio::fs::read_to_string(&issue_file_path).await.unwrap();
    assert!(initial_content.contains("> **Planning Mode**"));

    // Update to in-progress status
    update_issue(
        project_path,
        &created.id,
        UpdateIssueOptions {
            status: Some("in-progress".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("Should update issue");

    // Verify planning note was removed
    let updated_content = tokio::fs::read_to_string(&issue_file_path).await.unwrap();
    assert!(
        !updated_content.contains("> **Planning Mode**"),
        "Should remove planning note"
    );
    // With YAML frontmatter, content starts with --- not # Title
    assert!(
        updated_content.contains("# Test Issue"),
        "Should contain title"
    );
}

#[tokio::test]
async fn test_planning_note_idempotent() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue with planning status
    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test".to_string(),
            status: Some("planning".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // New format: {id}.md
    let issue_file_path = project_path
        .join(".centy/issues")
        .join(format!("{}.md", &created.id));

    // Update multiple times while staying in planning
    for i in 0..3 {
        update_issue(
            project_path,
            &created.id,
            UpdateIssueOptions {
                description: Some(format!("Updated description {i}")),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    }

    // Should still have exactly one planning note
    let content = tokio::fs::read_to_string(&issue_file_path).await.unwrap();
    assert_eq!(
        content.matches("> **Planning Mode**").count(),
        1,
        "Should have exactly one planning note"
    );
}

#[tokio::test]
async fn test_duplicate_issue_preserves_planning_note() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue with planning status
    let original = create_issue(
        project_path,
        CreateIssueOptions {
            title: "Planning Issue".to_string(),
            status: Some("planning".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Duplicate it
    let config = CentyConfig::default();
    let item_type_config = default_issue_config(&config);

    let result = generic_duplicate(
        &item_type_config,
        DuplicateGenericItemOptions {
            source_project_path: project_path.to_path_buf(),
            target_project_path: project_path.to_path_buf(),
            item_id: original.id.clone(),
            new_id: None,
            new_title: None,
        },
    )
    .await
    .expect("Should duplicate issue");

    // Verify duplicate has planning note (new format: {id}.md)
    let issue_file_path = project_path
        .join(".centy/issues")
        .join(format!("{}.md", &result.item.id));
    let content = tokio::fs::read_to_string(&issue_file_path).await.unwrap();

    assert!(
        content.contains("> **Planning Mode**"),
        "Duplicate should have planning note"
    );
    assert_eq!(result.item.frontmatter.status.as_deref(), Some("planning"));
}

#[tokio::test]
async fn test_planning_note_parsing_extracts_correct_title() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue with planning status
    let created = create_issue(
        project_path,
        CreateIssueOptions {
            title: "My Planning Issue".to_string(),
            description: "Some description".to_string(),
            status: Some("planning".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Get the issue and verify title is parsed correctly (not the planning note)
    let issue = get_issue(project_path, &created.id)
        .await
        .expect("Should get issue");
    assert_eq!(issue.title, "My Planning Issue");
    assert_eq!(issue.description, "Some description");
}
