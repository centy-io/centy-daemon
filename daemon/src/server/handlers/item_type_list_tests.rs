use super::builtin_item_types;
use crate::config::item_type_config::BUILTIN_ITEM_TYPE_FOLDERS;

/// The list fallback must name exactly the built-in folders, no more and no less.
#[tokio::test]
async fn test_builtin_item_types_match_builtin_folders() {
    let temp = tempfile::tempdir().expect("Failed to create temp directory");

    let mut listed: Vec<&str> = builtin_item_types(temp.path())
        .await
        .into_iter()
        .map(|(folder, _)| folder)
        .collect();
    listed.sort_unstable();

    let mut expected: Vec<&str> = BUILTIN_ITEM_TYPE_FOLDERS.to_vec();
    expected.sort_unstable();

    assert_eq!(
        listed, expected,
        "builtin_item_types must list exactly BUILTIN_ITEM_TYPE_FOLDERS"
    );
}
