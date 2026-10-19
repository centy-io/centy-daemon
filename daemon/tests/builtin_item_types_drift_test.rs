#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

mod common;

use centy_daemon::config::item_type_config::BUILTIN_ITEM_TYPE_FOLDERS;
use centy_daemon::server::handlers::item_type_resolve::resolve_item_type_config;
use common::{create_test_dir, init_centy_project};
use tokio::fs;

/// Guards drift between `BUILTIN_ITEM_TYPE_FOLDERS` and what `init` writes.
#[tokio::test]
async fn test_init_writes_config_for_exactly_the_builtin_folders() {
    let temp = create_test_dir();
    let path = temp.path();
    init_centy_project(path).await;

    let mut written = Vec::new();
    let mut entries = fs::read_dir(path.join(".centy"))
        .await
        .expect("Should read .centy");
    while let Some(entry) = entries.next_entry().await.expect("Should read entry") {
        if entry.path().join("config.yaml").is_file() {
            written.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    written.sort();

    let mut expected: Vec<String> = BUILTIN_ITEM_TYPE_FOLDERS
        .iter()
        .map(ToString::to_string)
        .collect();
    expected.sort();

    assert_eq!(
        written, expected,
        "init must write config.yaml for exactly BUILTIN_ITEM_TYPE_FOLDERS"
    );
}

/// Every built-in folder must resolve through the fallback when its config.yaml is gone.
#[tokio::test]
async fn test_every_builtin_folder_resolves_without_config_yaml() {
    for folder in BUILTIN_ITEM_TYPE_FOLDERS {
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
