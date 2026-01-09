use super::types::{ProjectRegistry, CURRENT_SCHEMA_VERSION};
use super::RegistryError;
use crate::utils::{atomic_write, now_iso};
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

/// Get the path to the global centy config directory (~/.centy)
pub fn get_centy_config_dir() -> Result<PathBuf, RegistryError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| RegistryError::HomeDirNotFound)?;

    Ok(PathBuf::from(home).join(".centy"))
}

/// Get the path to the global registry file (~/.centy/projects.json)
pub fn get_registry_path() -> Result<PathBuf, RegistryError> {
    Ok(get_centy_config_dir()?.join("projects.json"))
}

/// Migrate registry from v1 to v2 (add organizations support)
fn migrate_v1_to_v2(registry: &mut ProjectRegistry) {
    // v1 -> v2: Add empty organizations map (already has #[serde(default)])
    // Existing projects remain ungrouped (organization_slug = None by default)
    registry.schema_version = 2;
    registry.updated_at = now_iso();
    info!("Migrated registry from v1 to v2 (added organizations support)");
}

/// Apply any necessary migrations to bring registry to current schema version
fn apply_migrations(registry: &mut ProjectRegistry) -> bool {
    let mut migrated = false;

    if registry.schema_version < 2 {
        migrate_v1_to_v2(registry);
        migrated = true;
    }

    // Future migrations go here:
    // if registry.schema_version < 3 { migrate_v2_to_v3(registry); migrated = true; }

    migrated
}

/// Read the registry from disk, applying any necessary migrations
pub async fn read_registry() -> Result<ProjectRegistry, RegistryError> {
    let path = get_registry_path()?;

    if !path.exists() {
        return Ok(ProjectRegistry::new());
    }

    let content = fs::read_to_string(&path).await?;
    let mut registry: ProjectRegistry = serde_json::from_str(&content)?;

    // Apply migrations if needed
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

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Write atomically using tempfile crate (auto-cleanup on failure)
    let content = serde_json::to_string_pretty(registry)?;
    atomic_write(&path, &content).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_registry_path() {
        // This test will work if HOME or USERPROFILE is set
        let result = get_registry_path();
        if std::env::var("HOME").is_ok() || std::env::var("USERPROFILE").is_ok() {
            assert!(result.is_ok());
            let path = result.unwrap();
            assert!(path.ends_with("projects.json"));
            assert!(path.to_string_lossy().contains(".centy"));
        }
    }

    #[test]
    fn test_project_registry_new() {
        let registry = ProjectRegistry::new();
        assert_eq!(registry.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(registry.projects.is_empty());
        assert!(registry.organizations.is_empty());
        assert!(!registry.updated_at.is_empty());
    }
}
