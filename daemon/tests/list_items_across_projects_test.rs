#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod common;

use centy_daemon::registry::track_project;
use centy_daemon::server::handlers::item_create::create_item;
use centy_daemon::server::handlers::item_list_global::list_items_across_projects;
use centy_daemon::server::proto::{CreateItemRequest, ListItemsAcrossProjectsRequest};
use common::{create_test_dir, init_centy_project};
use std::collections::HashMap;
use std::path::Path;

async fn tracked_project_with_issues(dir: &Path, titles: &[&str]) -> String {
    init_centy_project(dir).await;
    let path = dir.to_string_lossy().to_string();
    track_project(&path).await.expect("track project");
    // The registry stores canonical paths (e.g. /private/var on macOS).
    let path = std::fs::canonicalize(dir)
        .expect("canonicalize")
        .to_string_lossy()
        .to_string();
    for title in titles {
        let resp = create_item(CreateItemRequest {
            project_path: path.clone(),
            item_type: "issues".to_string(),
            title: (*title).to_string(),
            body: String::new(),
            status: "open".to_string(),
            priority: 2,
            tags: vec![],
            custom_fields: HashMap::new(),
            projects: vec![],
        })
        .await
        .unwrap()
        .into_inner();
        assert!(resp.success, "create failed: {}", resp.error);
    }
    path
}

fn request(limit: u32, offset: u32) -> ListItemsAcrossProjectsRequest {
    ListItemsAcrossProjectsRequest {
        item_type: "issues".to_string(),
        limit,
        offset,
        filter: String::new(),
    }
}

/// Single test: the registry is process-global, so the cases share one setup.
#[tokio::test]
async fn test_list_items_across_projects() {
    let dir_a = create_test_dir();
    let dir_b = create_test_dir();
    let path_a = tracked_project_with_issues(dir_a.path(), &["a-one", "a-two"]).await;
    let path_b = tracked_project_with_issues(dir_b.path(), &["b-one"]).await;

    // All items from both projects, each tagged with its source project.
    let all = list_items_across_projects(request(0, 0))
        .await
        .unwrap()
        .into_inner();
    assert!(all.success, "list failed: {}", all.error);
    assert_eq!(all.total_count, 3i32);
    assert_eq!(all.items.len(), 3);
    assert_eq!(
        all.items
            .iter()
            .filter(|i| i.project_path == path_a)
            .count(),
        2
    );
    assert_eq!(
        all.items
            .iter()
            .filter(|i| i.project_path == path_b)
            .count(),
        1
    );

    // Newest first across projects.
    let created: Vec<String> = all
        .items
        .iter()
        .map(|i| {
            i.item
                .as_ref()
                .unwrap()
                .metadata
                .as_ref()
                .unwrap()
                .created_at
                .clone()
        })
        .collect();
    let mut sorted = created.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(created, sorted);

    // Global pagination: total_count ignores limit/offset.
    let page = list_items_across_projects(request(1, 1))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.total_count, 3i32);
    assert_eq!(page.items.len(), 1);
    assert_eq!(
        page.items[0].item.as_ref().unwrap().id,
        all.items[1].item.as_ref().unwrap().id
    );

    // Offset past the end yields no items.
    let empty = list_items_across_projects(request(10, 99))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(empty.items.len(), 0usize);
    assert_eq!(empty.total_count, 3i32);
}
