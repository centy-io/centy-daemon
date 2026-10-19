#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

mod common;

use centy_daemon::server::handlers::item_type_list::list_item_types;
use centy_daemon::server::proto::ListItemTypesRequest;
use common::{create_test_dir, init_centy_project};
use tokio::fs;

/// A built-in type whose config.yaml is gone must still be listed, once.
#[tokio::test]
async fn test_list_includes_builtin_type_with_missing_config() {
    let temp = create_test_dir();
    let path = temp.path();
    init_centy_project(path).await;
    fs::remove_file(path.join(".centy").join("docs").join("config.yaml"))
        .await
        .expect("Should delete docs config.yaml");

    let resp = list_item_types(ListItemTypesRequest {
        project_path: path.to_str().unwrap().to_string(),
    })
    .await
    .unwrap()
    .into_inner();

    assert!(resp.success, "list failed: {}", resp.error);
    let plurals: Vec<&str> = resp.item_types.iter().map(|t| t.plural.as_str()).collect();
    assert!(
        plurals.contains(&"docs"),
        "docs should be listed, got {plurals:?}"
    );
    for plural in &plurals {
        let count = plurals.iter().filter(|p| *p == plural).count();
        assert_eq!(count, 1, "{plural} listed {count} times");
    }
}
