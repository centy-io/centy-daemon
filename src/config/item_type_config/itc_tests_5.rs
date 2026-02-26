use super::*;
use tempfile::tempdir;
use tokio::fs;

// ─── migrate tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_migrate_creates_both_configs() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");
    fs::create_dir_all(centy_dir.join("issues"))
        .await
        .expect("create issues/");
    fs::create_dir_all(centy_dir.join("docs"))
        .await
        .expect("create docs/");
    fs::create_dir_all(centy_dir.join("archived"))
        .await
        .expect("create archived/");

    let config = CentyConfig::default();
    let created = migrate_to_item_type_configs(temp.path(), &config, None)
        .await
        .expect("Should migrate");

    assert_eq!(created.len(), 3);
    assert!(created.contains(&"issues/config.yaml".to_string()));
    assert!(created.contains(&"docs/config.yaml".to_string()));
    assert!(created.contains(&"archived/config.yaml".to_string()));

    assert!(centy_dir.join("issues").join("config.yaml").exists());
    assert!(centy_dir.join("docs").join("config.yaml").exists());
    assert!(centy_dir.join("archived").join("config.yaml").exists());
}

#[tokio::test]
async fn test_migrate_skips_existing_configs() {
    let temp = tempdir().expect("Should create temp dir");
    let centy_dir = temp.path().join(".centy");
    fs::create_dir_all(centy_dir.join("issues"))
        .await
        .expect("create issues/");
    fs::create_dir_all(centy_dir.join("docs"))
        .await
        .expect("create docs/");
    fs::create_dir_all(centy_dir.join("archived"))
        .await
        .expect("create archived/");

    fs::write(
        centy_dir.join("issues").join("config.yaml"),
        "name: CustomIssue\nidentifier: uuid\nfeatures:\n  displayNumber: false\n  status: false\n  priority: false\n  softDelete: false\n  assets: false\n  orgSync: false\n  move: false\n  duplicate: false\n",
    )
    .await
    .expect("write");

    let config = CentyConfig::default();
    let created = migrate_to_item_type_configs(temp.path(), &config, None)
        .await
        .expect("Should migrate");

    assert_eq!(created.len(), 2);
    assert!(created.contains(&"docs/config.yaml".to_string()));
    assert!(created.contains(&"archived/config.yaml".to_string()));

    let content = fs::read_to_string(centy_dir.join("issues").join("config.yaml"))
        .await
        .expect("read");
    assert!(content.contains("name: CustomIssue"));
}
