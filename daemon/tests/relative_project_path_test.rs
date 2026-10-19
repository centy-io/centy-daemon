#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Handlers must reject a relative `project_path` instead of resolving it
//! against the daemon's working directory.

mod common;

use centy_daemon::server::handlers::config::get_config;
use centy_daemon::server::handlers::init::init;
use centy_daemon::server::handlers::item_list::list_items;
use centy_daemon::server::proto::{GetConfigRequest, InitRequest, ListItemsRequest};

const RELATIVE: &str = "relative-project-path-test/not-absolute";

#[tokio::test]
async fn test_init_rejects_relative_path_without_touching_disk() {
    let _dir = common::create_test_dir();
    let resp = init(InitRequest {
        project_path: RELATIVE.to_string(),
        ..Default::default()
    })
    .await
    .unwrap()
    .into_inner();
    assert!(!resp.success);
    assert!(resp.error.contains("INVALID_PATH"), "{}", resp.error);
    assert!(!std::path::Path::new(RELATIVE).exists());
}

#[tokio::test]
async fn test_get_config_rejects_relative_path() {
    let _dir = common::create_test_dir();
    let resp = get_config(GetConfigRequest {
        project_path: RELATIVE.to_string(),
    })
    .await
    .unwrap()
    .into_inner();
    assert!(!resp.success);
    assert!(resp.error.contains("INVALID_PATH"), "{}", resp.error);
}

#[tokio::test]
async fn test_list_items_rejects_relative_path() {
    let _dir = common::create_test_dir();
    let resp = list_items(ListItemsRequest {
        project_path: RELATIVE.to_string(),
        item_type: "issues".to_string(),
        ..Default::default()
    })
    .await
    .unwrap()
    .into_inner();
    assert!(!resp.success);
    assert!(resp.error.contains("INVALID_PATH"), "{}", resp.error);
}
