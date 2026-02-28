use super::types::{ProjectRegistry, CURRENT_SCHEMA_VERSION};
use super::RegistryError;
use crate::utils::now_iso;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::fs;
use tokio::sync::Mutex;
use tracing::info;

/// Global mutex for registry file access
static REGISTRY_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn get_lock() -> &'static Mutex<()> {
    REGISTRY_LOCK.get_or_init(|| Mutex::new(()))
}

/// Get the path to the global centy config directory (~/.centy).
///
/// If `CENTY_HOME` is set, that directory is used instead of `~/.centy`.
/// This allows tests and CI to use an isolated registry without touching
/// the user's real `~/.centy` data.
pub fn get_centy_config_dir() -> Result<PathBuf, RegistryError> {
    if let Ok(centy_home) = std::env::var("CENTY_HOME") {
        return Ok(PathBuf::from(centy_home));
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| RegistryError::HomeDirNotFound)?;
    Ok(PathBuf::from(home).join(".centy"))
}

/// Get the path to the global registry file (~/.centy/projects.json)
pub fn get_registry_path() -> Result<PathBuf, RegistryError> {
    Ok(get_centy_config_dir()?.join("projects.json"))
}

fn migrate_v1_to_v2(registry: &mut ProjectRegistry) {
    registry.schema_version = 2;
    registry.updated_at = now_iso();
    info!("Migrated registry from v1 to v2 (added organizations support)");
}

fn apply_migrations(registry: &mut ProjectRegistry) -> bool {
    let mut migrated = false;
    if registry.schema_version < 2 {
        migrate_v1_to_v2(registry);
        migrated = true;
    }
    migrated
}

/// Read the registry from disk, applying any necessary migrations
#[allow(unknown_lints, max_nesting_depth)]
pub async fn read_registry() -> Result<ProjectRegistry, RegistryError> {
    let path = get_registry_path()?;
    if !path.exists() {
        return Ok(ProjectRegistry::new());
    }
    let content = fs::read_to_string(&path).await?;
    let mut registry: ProjectRegistry = serde_json::from_str(&content)?;
    if registry.schema_version < CURRENT_SCHEMA_VERSION {
        let _guard = get_lock().lock().await;
        if apply_migrations(&mut registry) {
            write_registry_unlocked(&registry).await?;
        }
    }
    Ok(registry)
}

/// Write the registry to disk with locking and atomic write
#[allow(dead_code)]
pub async fn write_registry(registry: &ProjectRegistry) -> Result<(), RegistryError> {
    let _guard = get_lock().lock().await;
    write_registry_unlocked(registry).await
}

/// Write the registry to disk without acquiring the lock (caller must hold lock)
pub async fn write_registry_unlocked(registry: &ProjectRegistry) -> Result<(), RegistryError> {
    let path = get_registry_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Write atomically using a per-call unique temp file + rename.
    // Using a unique suffix avoids races when multiple processes write concurrently
    // (e.g. parallel integration-test binaries), which would otherwise cause one
    // process to rename a temp file written by another.
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let temp_name = format!("projects.{unique_id}.json.tmp");
    let temp_path = path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join(temp_name);
    let content = serde_json::to_string_pretty(registry)?;
    fs::write(&temp_path, &content).await?;
    fs::rename(&temp_path, &path).await?;
    Ok(())
}

#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;
