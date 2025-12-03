mod types;

pub use types::{CentyManifest, ManagedFileType};

use crate::utils::{get_manifest_path, now_iso, CENTY_VERSION};
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("Failed to read manifest: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse manifest: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Manifest not found at {0}")]
    NotFound(String),
}

/// Read the manifest from the project path
pub async fn read_manifest(project_path: &Path) -> Result<Option<CentyManifest>, ManifestError> {
    let manifest_path = get_manifest_path(project_path);

    if !manifest_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&manifest_path).await?;
    let manifest: CentyManifest = serde_json::from_str(&content)?;
    Ok(Some(manifest))
}

/// Write the manifest to the project path
pub async fn write_manifest(
    project_path: &Path,
    manifest: &CentyManifest,
) -> Result<(), ManifestError> {
    let manifest_path = get_manifest_path(project_path);
    let content = serde_json::to_string_pretty(manifest)?;
    fs::write(&manifest_path, content).await?;
    Ok(())
}

/// Create a new empty manifest
pub fn create_manifest() -> CentyManifest {
    let now = now_iso();
    CentyManifest {
        schema_version: 1,
        centy_version: CENTY_VERSION.to_string(),
        created_at: now.clone(),
        updated_at: now,
    }
}

/// Update the manifest timestamp
pub fn update_manifest_timestamp(manifest: &mut CentyManifest) {
    manifest.updated_at = now_iso();
}

#[cfg(test)]
mod tests {
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
}
