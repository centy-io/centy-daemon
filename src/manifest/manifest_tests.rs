use super::*;

#[test]
fn test_create_manifest() {
    let manifest = create_manifest();

    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.centy_version, CENTY_VERSION);
    assert!(!manifest.created_at.is_empty());
    assert!(!manifest.updated_at.is_empty());
}

#[test]
fn test_update_manifest_sets_version() {
    let mut manifest = CentyManifest {
        schema_version: 1,
        centy_version: "0.0.0".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };

    update_manifest(&mut manifest);

    assert_eq!(manifest.centy_version, CENTY_VERSION);
    assert_ne!(manifest.updated_at, "2024-01-01T00:00:00Z");
}

#[test]
fn test_manifest_serialization() {
    let manifest = create_manifest();

    // Serialize
    let json = serde_json::to_string(&manifest).expect("Should serialize");

    // Deserialize
    let deserialized: CentyManifest = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(manifest.schema_version, deserialized.schema_version);
    assert_eq!(manifest.centy_version, deserialized.centy_version);
}

#[test]
fn test_manifest_json_uses_camel_case() {
    let manifest = create_manifest();
    let json = serde_json::to_string(&manifest).expect("Should serialize");

    // Check for camelCase keys
    assert!(json.contains("schemaVersion"));
    assert!(json.contains("centyVersion"));
    assert!(json.contains("createdAt"));
    assert!(json.contains("updatedAt"));

    // Should NOT contain snake_case
    assert!(!json.contains("schema_version"));
    assert!(!json.contains("centy_version"));
}

#[tokio::test]
async fn test_write_and_read_manifest() {
    use tempfile::tempdir;

    let manifest = create_manifest();

    // Create temp directory and write manifest
    let temp_dir = tempdir().expect("Should create temp dir");
    let centy_dir = temp_dir.path().join(".centy");
    fs::create_dir_all(&centy_dir)
        .await
        .expect("Should create .centy dir");

    write_manifest(temp_dir.path(), &manifest)
        .await
        .expect("Should write manifest");

    // Read back and verify
    let read_manifest = read_manifest(temp_dir.path())
        .await
        .expect("Should read manifest")
        .expect("Manifest should exist");

    assert_eq!(read_manifest.schema_version, manifest.schema_version);
    assert_eq!(read_manifest.centy_version, manifest.centy_version);
}
