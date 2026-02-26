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
}

/// Read the manifest from the project path
pub async fn read_manifest(project_path: &Path) -> Result<Option<CentyManifest>, ManifestError> {
    let manifest_path = get_manifest_path(project_path);
    if !manifest_path.exists() { return Ok(None); }
    let content = fs::read_to_string(&manifest_path).await?;
    let manifest: CentyManifest = serde_json::from_str(&content)?;
    Ok(Some(manifest))
}

/// Write the manifest to the project path
pub async fn write_manifest(project_path: &Path, manifest: &CentyManifest) -> Result<(), ManifestError> {
    let manifest_path = get_manifest_path(project_path);
    let content = serde_json::to_string_pretty(manifest)?;
    fs::write(&manifest_path, content).await?;
    Ok(())
}

/// Create a new empty manifest
#[must_use]
pub fn create_manifest() -> CentyManifest {
    let now = now_iso();
    CentyManifest { schema_version: 1, centy_version: CENTY_VERSION.to_string(), created_at: now.clone(), updated_at: now }
}

/// Update the manifest timestamp and version
pub fn update_manifest(manifest: &mut CentyManifest) {
    manifest.updated_at = now_iso();
    manifest.centy_version = CENTY_VERSION.to_string();
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
