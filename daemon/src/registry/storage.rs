use super::migrations::apply_migrations;
use super::types::{ProjectRegistry, CURRENT_SCHEMA_VERSION};
use super::RegistryError;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::fs;
use tokio::sync::Mutex;

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
        .map_err(|_e| RegistryError::HomeDirNotFound)?;
    Ok(PathBuf::from(home).join(".centy"))
}

/// Get the path to the global registry file (~/.centy/projects.json)
pub fn get_registry_path() -> Result<PathBuf, RegistryError> {
    Ok(get_centy_config_dir()?.join("projects.json"))
}

/// Read the registry from disk, applying any necessary migrations
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

/// Write the registry to disk without acquiring the lock (caller must hold lock)
pub async fn write_registry_unlocked(registry: &ProjectRegistry) -> Result<(), RegistryError> {
    let path = get_registry_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Write atomically: a uniquely named temp file in the same directory, then
    // rename over the target. `NamedTempFile` removes the temp file on any
    // failure, so no stale `*.tmp` files are left behind.
    let content = serde_json::to_string_pretty(registry)?;
    let dir = path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    tokio::task::spawn_blocking(move || -> std::io::Result<()> {
        use std::io::Write as _;
        let mut temp = tempfile::NamedTempFile::new_in(&dir)?;
        temp.write_all(content.as_bytes())?;
        temp.persist(&path).map(drop).map_err(|e| e.error)
    })
    .await
    .map_err(std::io::Error::other)??;
    Ok(())
}

/// Shared process-wide test lock so that all registry unit tests in this
/// library binary are serialized and share a single stable `CENTY_HOME`.
///
/// All test modules that read/write the registry **must** acquire this lock
/// (via `acquire_registry_test_lock()`) rather than managing `CENTY_HOME`
/// themselves. This prevents races when multiple modules run concurrently.
#[cfg(test)]
#[allow(dead_code)]
pub static REGISTRY_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Single `CENTY_HOME` `TempDir` initialized once per test-binary run.
#[cfg(test)]
#[allow(dead_code)]
pub static REGISTRY_TEST_HOME: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();

/// Acquire the shared registry test lock, initializing `CENTY_HOME` if needed.
#[cfg(test)]
pub fn acquire_registry_test_lock() -> std::sync::MutexGuard<'static, ()> {
    REGISTRY_TEST_HOME.get_or_init(|| {
        let dir = tempfile::TempDir::new().expect("registry test home dir");
        std::env::set_var("CENTY_HOME", dir.path());
        dir
    });
    REGISTRY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;
