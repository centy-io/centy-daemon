mod common;

use centy_daemon::item::entities::pr::{
    create_pr, delete_pr, get_pr, get_pr_by_display_number, list_prs, update_pr,
    CreatePrOptions, UpdatePrOptions,
};
use centy_daemon::item::entities::pr::create::PrError;
use centy_daemon::item::entities::pr::crud::PrCrudError;
use centy_daemon::item::entities::issue::is_uuid;
use common::{create_test_dir, init_centy_project};
use std::collections::HashMap;

#[tokio::test]
async fn test_create_pr_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let options = CreatePrOptions {
        title: "Add new feature".to_string(),
        description: "This PR adds a new feature".to_string(),
        source_branch: Some("feature/new-feature".to_string()),
        target_branch: Some("main".to_string()),
        priority: Some(1),
        status: Some("draft".to_string()),
        reviewers: vec!["reviewer1".to_string()],
        custom_fields: HashMap::new(),
        ..Default::default()
    };

    let result = create_pr(project_path, options)
        .await
        .expect("Should create PR");

    assert!(is_uuid(&result.id), "PR ID should be a UUID");
    assert_eq!(result.display_number, 1);
    assert_eq!(result.created_files.len(), 3);
    assert_eq!(result.detected_source_branch, "feature/new-feature");

    let pr_path = project_path.join(format!(".centy/prs/{}", result.id));
    assert!(pr_path.join("pr.md").exists());
    assert!(pr_path.join("metadata.json").exists());
    assert!(pr_path.join("assets").exists());
}

#[tokio::test]
async fn test_create_pr_increments_display_number() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let options1 = CreatePrOptions {
        title: "First PR".to_string(),
        source_branch: Some("feature/first".to_string()),
        ..Default::default()
    };
    let result1 = create_pr(project_path, options1).await.expect("Should create");
    assert_eq!(result1.display_number, 1);

    let options2 = CreatePrOptions {
        title: "Second PR".to_string(),
        source_branch: Some("feature/second".to_string()),
        ..Default::default()
    };
    let result2 = create_pr(project_path, options2).await.expect("Should create");
    assert_eq!(result2.display_number, 2);

    assert_ne!(result1.id, result2.id);
}

#[tokio::test]
async fn test_create_pr_requires_init() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    let options = CreatePrOptions {
        title: "Test PR".to_string(),
        source_branch: Some("feature/test".to_string()),
        ..Default::default()
    };

    let result = create_pr(project_path, options).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), PrError::NotInitialized));
}

#[tokio::test]
async fn test_create_pr_requires_title() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let options = CreatePrOptions {
        title: String::new(),
        source_branch: Some("feature/test".to_string()),
        ..Default::default()
    };

    let result = create_pr(project_path, options).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), PrError::TitleRequired));
}

#[tokio::test]
async fn test_get_pr_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let options = CreatePrOptions {
        title: "My Test PR".to_string(),
        description: "Description here".to_string(),
        source_branch: Some("feature/test".to_string()),
        target_branch: Some("main".to_string()),
        priority: Some(1),
        status: Some("open".to_string()),
        reviewers: vec!["alice".to_string(), "bob".to_string()],
        ..Default::default()
    };
    let result = create_pr(project_path, options).await.expect("Should create");

    let pr = get_pr(project_path, &result.id).await.expect("Should get PR");

    assert!(is_uuid(&pr.id), "PR ID should be a UUID");
    assert_eq!(pr.metadata.display_number, 1);
    assert_eq!(pr.title, "My Test PR");
    assert_eq!(pr.description, "Description here");
    assert_eq!(pr.metadata.priority, 1);
    assert_eq!(pr.metadata.status, "open");
}

#[tokio::test]
async fn test_get_pr_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let result = get_pr(project_path, "nonexistent-uuid").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), PrCrudError::PrNotFound(_)));
}

#[tokio::test]
async fn test_get_pr_by_display_number() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let options1 = CreatePrOptions {
        title: "First PR".to_string(),
        source_branch: Some("feature/first".to_string()),
        ..Default::default()
    };
    create_pr(project_path, options1).await.expect("Should create");

    let options2 = CreatePrOptions {
        title: "Second PR".to_string(),
        source_branch: Some("feature/second".to_string()),
        ..Default::default()
    };
    create_pr(project_path, options2).await.expect("Should create");

    let pr = get_pr_by_display_number(project_path, 2).await.expect("Should get PR");
    assert_eq!(pr.title, "Second PR");
    assert_eq!(pr.metadata.display_number, 2);
}

#[tokio::test]
async fn test_list_prs_empty() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let prs = list_prs(project_path, None, None, None, None, false)
        .await
        .expect("Should list PRs");

    assert!(prs.is_empty());
}

#[tokio::test]
async fn test_list_prs_returns_all() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    for i in 1..=3 {
        let options = CreatePrOptions {
            title: format!("PR {i}"),
            source_branch: Some(format!("feature/pr-{i}")),
            ..Default::default()
        };
        create_pr(project_path, options).await.expect("Should create");
    }

    let prs = list_prs(project_path, None, None, None, None, false)
        .await
        .expect("Should list PRs");

    assert_eq!(prs.len(), 3);
    assert_eq!(prs[0].metadata.display_number, 1);
    assert_eq!(prs[1].metadata.display_number, 2);
    assert_eq!(prs[2].metadata.display_number, 3);
}

#[tokio::test]
async fn test_list_prs_filter_by_status() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let options1 = CreatePrOptions {
        title: "Open PR".to_string(),
        source_branch: Some("feature/open".to_string()),
        status: Some("open".to_string()),
        ..Default::default()
    };
    create_pr(project_path, options1).await.expect("Should create");

    let options2 = CreatePrOptions {
        title: "Draft PR".to_string(),
        source_branch: Some("feature/draft".to_string()),
        status: Some("draft".to_string()),
        ..Default::default()
    };
    create_pr(project_path, options2).await.expect("Should create");

    let open_prs = list_prs(project_path, Some("open"), None, None, None, false)
        .await
        .expect("Should list PRs");

    assert_eq!(open_prs.len(), 1);
    assert_eq!(open_prs[0].metadata.status, "open");
}

#[tokio::test]
async fn test_update_pr_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let options = CreatePrOptions {
        title: "Original Title".to_string(),
        description: "Original description".to_string(),
        source_branch: Some("feature/test".to_string()),
        ..Default::default()
    };
    let result = create_pr(project_path, options).await.expect("Should create");

    let update_options = UpdatePrOptions {
        title: Some("Updated Title".to_string()),
        description: Some("Updated description".to_string()),
        ..Default::default()
    };

    let update_result = update_pr(project_path, &result.id, update_options)
        .await
        .expect("Should update");

    assert_eq!(update_result.pr.title, "Updated Title");
    assert_eq!(update_result.pr.description, "Updated description");
}

#[tokio::test]
async fn test_update_pr_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let update_options = UpdatePrOptions {
        title: Some("New Title".to_string()),
        ..Default::default()
    };

    let result = update_pr(project_path, "nonexistent-uuid", update_options).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), PrCrudError::PrNotFound(_)));
}

#[tokio::test]
async fn test_delete_pr_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let options = CreatePrOptions {
        title: "PR to delete".to_string(),
        source_branch: Some("feature/test".to_string()),
        ..Default::default()
    };
    let result = create_pr(project_path, options).await.expect("Should create");

    let pr_path = project_path.join(format!(".centy/prs/{}", result.id));
    assert!(pr_path.exists());

    delete_pr(project_path, &result.id).await.expect("Should delete");

    assert!(!pr_path.exists());
}

#[tokio::test]
async fn test_delete_pr_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let result = delete_pr(project_path, "nonexistent-uuid").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), PrCrudError::PrNotFound(_)));
}
