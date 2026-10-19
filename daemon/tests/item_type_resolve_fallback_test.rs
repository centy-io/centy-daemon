#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

mod common;

use centy_daemon::server::handlers::item_type_resolve::resolve_item_type_config;
use common::{create_test_dir, init_centy_project};
use tokio::fs;

/// Every type init creates must still resolve when its config.yaml is gone.
#[tokio::test]
async fn test_resolve_falls_back_for_all_default_types() {
    for folder in [
        "issues",
        "docs",
        "archived",
        "comments",
        "user_stories",
        "epics",
    ] {
        let temp = create_test_dir();
        let path = temp.path();
        init_centy_project(path).await;
        fs::remove_file(path.join(".centy").join(folder).join("config.yaml"))
            .await
            .expect("Should delete config.yaml");

        let (resolved, _) = resolve_item_type_config(path, folder)
            .await
            .unwrap_or_else(|e| panic!("{folder} should resolve via fallback: {e}"));
        assert_eq!(resolved, folder);
    }
}
