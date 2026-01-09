//! Migration executor for running migrations.

use super::registry::MigrationRegistry;
use super::types::{Migration, MigrationDirection, MigrationError, MigrationResult};
use crate::config::{read_config, write_config};
use crate::version::SemVer;
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info};

/// Executor for running migrations.
///
/// The executor takes a registry of migrations and provides methods
/// to migrate projects between versions.
pub struct MigrationExecutor {
    registry: Arc<MigrationRegistry>,
}

impl MigrationExecutor {
    /// Create a new executor with the given registry.
    #[must_use]
    pub fn new(registry: Arc<MigrationRegistry>) -> Self {
        Self { registry }
    }

    /// Execute migrations to reach the target version.
    ///
    /// This method:
    /// 1. Reads the current project version from config
    /// 2. Finds the migration path to the target version
    /// 3. Executes each migration in order
    /// 4. Rolls back on failure
    /// 5. Updates the config with the new version on success
    #[allow(clippy::too_many_lines)]
    pub async fn migrate(
        &self,
        project_path: &Path,
        target_version: &SemVer,
    ) -> Result<MigrationResult, MigrationError> {
        // Read current config to get project version
        let config = read_config(project_path)
            .await
            .map_err(|e| MigrationError::ConfigError(e.to_string()))?;

        let current_version = config
            .as_ref()
            .and_then(|c| c.version.as_ref())
            .map(|v| SemVer::parse(v))
            .transpose()?
            .unwrap_or_else(|| SemVer::new(0, 0, 0)); // Unversioned projects start at 0.0.0

        info!(
            from = %current_version,
            to = %target_version,
            "Starting migration"
        );

        // Get migration path
        let (migrations, direction) = self
            .registry
            .get_migration_path(&current_version, target_version)?;

        if migrations.is_empty() {
            info!("No migrations needed, already at target version");
            return Ok(MigrationResult {
                success: true,
                from_version: current_version.to_string(),
                to_version: target_version.to_string(),
                migrations_applied: vec![],
                error: None,
            });
        }

        let mut applied: Vec<Arc<dyn Migration>> = Vec::new();

        // Execute migrations
        for migration in &migrations {
            let migration_name = format!(
                "{} -> {}: {}",
                migration.from_version(),
                migration.to_version(),
                migration.description()
            );

            info!(migration = %migration_name, "Applying migration");

            let result = match direction {
                MigrationDirection::Up => migration.up(project_path).await,
                MigrationDirection::Down => migration.down(project_path).await,
            };

            if let Err(e) = result {
                error!(migration = %migration_name, error = %e, "Migration failed");

                // Rollback applied migrations
                for applied_migration in applied.iter().rev() {
                    let rollback_name = format!(
                        "{} -> {}",
                        applied_migration.from_version(),
                        applied_migration.to_version()
                    );
                    info!(migration = %rollback_name, "Rolling back migration");

                    if let Err(rollback_err) = self
                        .rollback_migration(project_path, applied_migration, direction)
                        .await
                    {
                        error!(
                            migration = %rollback_name,
                            error = %rollback_err,
                            "Rollback failed"
                        );
                    }
                }

                return Ok(MigrationResult {
                    success: false,
                    from_version: current_version.to_string(),
                    to_version: target_version.to_string(),
                    migrations_applied: vec![],
                    error: Some(format!("Migration {migration_name} failed: {e}")),
                });
            }

            applied.push(Arc::clone(migration));
        }

        // Update config with new version
        let mut config = config.unwrap_or_default();
        config.version = Some(target_version.to_string());
        write_config(project_path, &config)
            .await
            .map_err(|e| MigrationError::ConfigError(e.to_string()))?;

        info!(
            from = %current_version,
            to = %target_version,
            count = applied.len(),
            "Migration completed successfully"
        );

        Ok(MigrationResult {
            success: true,
            from_version: current_version.to_string(),
            to_version: target_version.to_string(),
            migrations_applied: applied
                .iter()
                .map(|m| {
                    format!(
                        "{} -> {}: {}",
                        m.from_version(),
                        m.to_version(),
                        m.description()
                    )
                })
                .collect(),
            error: None,
        })
    }

    /// Rollback a single migration.
    async fn rollback_migration(
        &self,
        project_path: &Path,
        migration: &Arc<dyn Migration>,
        direction: MigrationDirection,
    ) -> Result<(), MigrationError> {
        // To rollback, we do the opposite operation
        match direction {
            MigrationDirection::Up => migration.down(project_path).await,
            MigrationDirection::Down => migration.up(project_path).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    // Mock migration for testing
    struct MockMigration {
        from: SemVer,
        to: SemVer,
        description: &'static str,
    }

    #[async_trait]
    impl Migration for MockMigration {
        fn from_version(&self) -> &SemVer {
            &self.from
        }

        fn to_version(&self) -> &SemVer {
            &self.to
        }

        fn description(&self) -> &'static str {
            self.description
        }

        async fn up(&self, _project_path: &Path) -> Result<(), MigrationError> {
            Ok(())
        }

        async fn down(&self, _project_path: &Path) -> Result<(), MigrationError> {
            Ok(())
        }
    }

    // Failing migration for testing rollback
    struct FailingMigration {
        from: SemVer,
        to: SemVer,
    }

    #[async_trait]
    impl Migration for FailingMigration {
        fn from_version(&self) -> &SemVer {
            &self.from
        }

        fn to_version(&self) -> &SemVer {
            &self.to
        }

        fn description(&self) -> &'static str {
            "Failing migration"
        }

        async fn up(&self, _project_path: &Path) -> Result<(), MigrationError> {
            Err(MigrationError::ConfigError(
                "intentional failure".to_string(),
            ))
        }

        async fn down(&self, _project_path: &Path) -> Result<(), MigrationError> {
            Ok(())
        }
    }

    #[test]
    fn test_migration_executor_new() {
        let registry = MigrationRegistry::new();
        let executor = MigrationExecutor::new(Arc::new(registry));

        // Should not panic - executor created successfully
        let _ = executor;
    }

    #[tokio::test]
    async fn test_migrate_no_changes_needed() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_path = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_path)
            .await
            .expect("Should create dir");

        // Write config with version 1.0.0
        let config = r#"{"version": "1.0.0"}"#;
        fs::write(centy_path.join("config.json"), config)
            .await
            .expect("Should write config");

        let registry = MigrationRegistry::new();
        let executor = MigrationExecutor::new(Arc::new(registry));

        let result = executor
            .migrate(temp_dir.path(), &SemVer::new(1, 0, 0))
            .await
            .expect("Should migrate");

        assert!(result.success);
        assert!(result.migrations_applied.is_empty());
        assert_eq!(result.from_version, "1.0.0");
        assert_eq!(result.to_version, "1.0.0");
    }

    #[tokio::test]
    async fn test_migrate_single_upgrade() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_path = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_path)
            .await
            .expect("Should create dir");

        // Start with no version (will be 0.0.0)
        let config = "{}";
        fs::write(centy_path.join("config.json"), config)
            .await
            .expect("Should write config");

        let mut registry = MigrationRegistry::new();
        registry.register(Arc::new(MockMigration {
            from: SemVer::new(0, 0, 0),
            to: SemVer::new(0, 1, 0),
            description: "Initial setup",
        }));

        let executor = MigrationExecutor::new(Arc::new(registry));

        let result = executor
            .migrate(temp_dir.path(), &SemVer::new(0, 1, 0))
            .await
            .expect("Should migrate");

        assert!(result.success);
        assert_eq!(result.migrations_applied.len(), 1);
        assert!(result.migrations_applied[0].contains("Initial setup"));
    }

    #[tokio::test]
    async fn test_migrate_updates_config_version() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_path = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_path)
            .await
            .expect("Should create dir");

        let config = "{}";
        fs::write(centy_path.join("config.json"), config)
            .await
            .expect("Should write config");

        let mut registry = MigrationRegistry::new();
        registry.register(Arc::new(MockMigration {
            from: SemVer::new(0, 0, 0),
            to: SemVer::new(0, 1, 0),
            description: "Test migration",
        }));

        let executor = MigrationExecutor::new(Arc::new(registry));

        executor
            .migrate(temp_dir.path(), &SemVer::new(0, 1, 0))
            .await
            .expect("Should migrate");

        // Read config back and verify version was updated
        let content = fs::read_to_string(centy_path.join("config.json"))
            .await
            .expect("Should read config");

        assert!(content.contains("0.1.0"));
    }

    #[tokio::test]
    async fn test_migrate_multi_step() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_path = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_path)
            .await
            .expect("Should create dir");

        let config = "{}";
        fs::write(centy_path.join("config.json"), config)
            .await
            .expect("Should write config");

        let mut registry = MigrationRegistry::new();
        registry.register(Arc::new(MockMigration {
            from: SemVer::new(0, 0, 0),
            to: SemVer::new(0, 1, 0),
            description: "Step 1",
        }));
        registry.register(Arc::new(MockMigration {
            from: SemVer::new(0, 1, 0),
            to: SemVer::new(0, 2, 0),
            description: "Step 2",
        }));

        let executor = MigrationExecutor::new(Arc::new(registry));

        let result = executor
            .migrate(temp_dir.path(), &SemVer::new(0, 2, 0))
            .await
            .expect("Should migrate");

        assert!(result.success);
        assert_eq!(result.migrations_applied.len(), 2);
    }

    #[tokio::test]
    async fn test_migrate_fails_returns_error_result() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_path = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_path)
            .await
            .expect("Should create dir");

        let config = "{}";
        fs::write(centy_path.join("config.json"), config)
            .await
            .expect("Should write config");

        let mut registry = MigrationRegistry::new();
        registry.register(Arc::new(FailingMigration {
            from: SemVer::new(0, 0, 0),
            to: SemVer::new(0, 1, 0),
        }));

        let executor = MigrationExecutor::new(Arc::new(registry));

        let result = executor
            .migrate(temp_dir.path(), &SemVer::new(0, 1, 0))
            .await
            .expect("Should return result");

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("intentional failure"));
    }

    #[tokio::test]
    async fn test_migrate_no_path_available() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_path = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_path)
            .await
            .expect("Should create dir");

        let config = "{}";
        fs::write(centy_path.join("config.json"), config)
            .await
            .expect("Should write config");

        // Empty registry - no migrations available
        let registry = MigrationRegistry::new();
        let executor = MigrationExecutor::new(Arc::new(registry));

        let result = executor
            .migrate(temp_dir.path(), &SemVer::new(1, 0, 0))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_migrate_without_config_version() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_path = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_path)
            .await
            .expect("Should create dir");

        // Config exists but has no version field
        let config = r#"{"priorityLevels": 3}"#;
        fs::write(centy_path.join("config.json"), config)
            .await
            .expect("Should write config");

        let mut registry = MigrationRegistry::new();
        registry.register(Arc::new(MockMigration {
            from: SemVer::new(0, 0, 0),
            to: SemVer::new(0, 1, 0),
            description: "Test",
        }));

        let executor = MigrationExecutor::new(Arc::new(registry));

        // Should handle missing version gracefully (treats as 0.0.0)
        let result = executor
            .migrate(temp_dir.path(), &SemVer::new(0, 1, 0))
            .await
            .expect("Should migrate");

        assert!(result.success);
        assert_eq!(result.from_version, "0.0.0");
    }
}
