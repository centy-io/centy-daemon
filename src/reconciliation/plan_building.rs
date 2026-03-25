use super::*;

#[tokio::test]
async fn test_build_reconciliation_plan_empty_directory() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let plan = build_reconciliation_plan(temp_dir.path())
        .await
        .expect("Should build plan");
    assert!(!plan.to_create.is_empty());
    assert!(plan.to_restore.is_empty());
    assert!(plan.to_reset.is_empty());
    assert!(plan.up_to_date.is_empty());
    assert!(plan.user_files.is_empty());
}

#[tokio::test]
async fn test_build_reconciliation_plan_includes_all_managed_files() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let plan = build_reconciliation_plan(temp_dir.path())
        .await
        .expect("Should build plan");
    let managed_files = get_managed_files();
    assert_eq!(plan.to_create.len(), managed_files.len());
}

#[tokio::test]
async fn test_scan_centy_folder_empty() {
    use tempfile::tempdir;
    let temp_dir = tempdir().expect("Should create temp dir");
    let centy_path = temp_dir.path().join(".centy");
    let files = helpers::scan_centy_folder(&centy_path);
    assert!(files.is_empty());
}

#[tokio::test]
async fn test_scan_centy_folder_with_files() {
    use tempfile::tempdir;
    use tokio::fs;
    let temp_dir = tempdir().expect("Should create temp dir");
    let centy_path = temp_dir.path().join(".centy");
    fs::create_dir_all(&centy_path)
        .await
        .expect("Should create .centy");
    fs::write(centy_path.join("test.txt"), "content")
        .await
        .expect("Should write file");
    fs::create_dir_all(centy_path.join("subdir"))
        .await
        .expect("Should create subdir");
    let files = helpers::scan_centy_folder(&centy_path);
    assert!(files.contains("test.txt"));
    assert!(files.contains("subdir/"));
}

#[tokio::test]
async fn test_scan_centy_folder_skips_manifest() {
    use tempfile::tempdir;
    use tokio::fs;
    let temp_dir = tempdir().expect("Should create temp dir");
    let centy_path = temp_dir.path().join(".centy");
    fs::create_dir_all(&centy_path)
        .await
        .expect("Should create .centy");
    fs::write(centy_path.join(".centy-manifest.json"), "{}")
        .await
        .expect("Should write manifest");
    fs::write(centy_path.join("README.md"), "content")
        .await
        .expect("Should write readme");
    let files = helpers::scan_centy_folder(&centy_path);
    assert!(files.contains("README.md"));
    assert!(!files.contains(".centy-manifest.json"));
}

#[test]
fn test_reconciliation_plan_clone() {
    let mut plan = ReconciliationPlan::default();
    plan.to_create.push(FileInfo {
        path: "test.md".to_string(),
        file_type: ManagedFileType::File,
        hash: "abc".to_string(),
        content_preview: None,
    });
    let cloned = plan.clone();
    assert_eq!(cloned.to_create.len(), 1);
    assert_eq!(cloned.to_create[0].path, "test.md");
}

#[test]
fn test_reconciliation_plan_debug() {
    let plan = ReconciliationPlan::default();
    let debug_str = format!("{plan:?}");
    assert!(debug_str.contains("ReconciliationPlan"));
}
