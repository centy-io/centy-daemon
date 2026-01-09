//! Types for the migration system.

use async_trait::async_trait;
use semver::Version;
use std::path::Path;
use thiserror::Error;

/// Error types for migration operations.
#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Semver parse error: {0}")]
    SemverError(#[from] semver::Error),

    #[error("No migration path from {0} to {1}")]
    NoMigrationPath(String, String),

    #[error("Config error: {0}")]
    ConfigError(String),
}

/// Trait for a single migration.
///
/// Each migration represents a transformation from one version to another.
/// Migrations must be reversible (implement both up and down).
#[async_trait]
pub trait Migration: Send + Sync {
    /// The version this migration upgrades FROM.
    fn from_version(&self) -> &Version;

    /// The version this migration upgrades TO.
    fn to_version(&self) -> &Version;

    /// Human-readable description of what this migration does.
    fn description(&self) -> &str;

    /// Apply the migration (upgrade).
    async fn up(&self, project_path: &Path) -> Result<(), MigrationError>;

    /// Revert the migration (downgrade).
    async fn down(&self, project_path: &Path) -> Result<(), MigrationError>;
}

/// Result of migration execution.
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Whether the migration was successful.
    pub success: bool,
    /// The version we migrated from.
    pub from_version: String,
    /// The version we migrated to.
    pub to_version: String,
    /// List of migrations that were applied (descriptions).
    pub migrations_applied: Vec<String>,
    /// Error message if migration failed.
    pub error: Option<String>,
}

/// Direction of migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationDirection {
    /// Upgrading to a newer version.
    Up,
    /// Downgrading to an older version.
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_error_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let migration_err = MigrationError::IoError(io_err);

        let display = format!("{migration_err}");
        assert!(display.contains("IO error"));
    }

    #[test]
    fn test_migration_error_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json")
            .expect_err("Should be invalid");
        let migration_err = MigrationError::JsonError(json_err);

        let display = format!("{migration_err}");
        assert!(display.contains("JSON error"));
    }

    #[test]
    fn test_migration_error_no_migration_path() {
        let err = MigrationError::NoMigrationPath("0.1.0".to_string(), "1.0.0".to_string());
        let display = format!("{err}");

        assert!(display.contains("No migration path"));
        assert!(display.contains("0.1.0"));
        assert!(display.contains("1.0.0"));
    }

    #[test]
    fn test_migration_error_config_error() {
        let err = MigrationError::ConfigError("invalid config".to_string());
        let display = format!("{err}");

        assert!(display.contains("Config error"));
        assert!(display.contains("invalid config"));
    }

    #[test]
    fn test_migration_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let migration_err = MigrationError::from(io_err);

        assert!(matches!(migration_err, MigrationError::IoError(_)));
    }

    #[test]
    fn test_migration_error_from_json_error() {
        let json_err = serde_json::from_str::<String>("123").expect_err("Should fail");
        let migration_err = MigrationError::from(json_err);

        assert!(matches!(migration_err, MigrationError::JsonError(_)));
    }

    #[test]
    fn test_migration_result_initialization() {
        let result = MigrationResult {
            success: true,
            from_version: "0.1.0".to_string(),
            to_version: "1.0.0".to_string(),
            migrations_applied: vec!["Migration 1".to_string(), "Migration 2".to_string()],
            error: None,
        };

        assert!(result.success);
        assert_eq!(result.from_version, "0.1.0");
        assert_eq!(result.to_version, "1.0.0");
        assert_eq!(result.migrations_applied.len(), 2);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_migration_result_with_error() {
        let result = MigrationResult {
            success: false,
            from_version: "0.1.0".to_string(),
            to_version: "1.0.0".to_string(),
            migrations_applied: vec![],
            error: Some("Migration failed".to_string()),
        };

        assert!(!result.success);
        assert!(result.error.is_some());
        assert_eq!(result.error.unwrap(), "Migration failed");
    }

    #[test]
    fn test_migration_result_clone() {
        let result = MigrationResult {
            success: true,
            from_version: "0.1.0".to_string(),
            to_version: "1.0.0".to_string(),
            migrations_applied: vec!["test".to_string()],
            error: None,
        };

        let cloned = result.clone();
        assert_eq!(cloned.success, result.success);
        assert_eq!(cloned.from_version, result.from_version);
        assert_eq!(cloned.migrations_applied, result.migrations_applied);
    }

    #[test]
    fn test_migration_result_debug() {
        let result = MigrationResult {
            success: true,
            from_version: "0.1.0".to_string(),
            to_version: "1.0.0".to_string(),
            migrations_applied: vec![],
            error: None,
        };

        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("MigrationResult"));
        assert!(debug_str.contains("success"));
    }

    #[test]
    fn test_migration_direction_up() {
        let direction = MigrationDirection::Up;
        assert_eq!(direction, MigrationDirection::Up);
        assert_ne!(direction, MigrationDirection::Down);
    }

    #[test]
    fn test_migration_direction_down() {
        let direction = MigrationDirection::Down;
        assert_eq!(direction, MigrationDirection::Down);
        assert_ne!(direction, MigrationDirection::Up);
    }

    #[test]
    fn test_migration_direction_clone() {
        let direction = MigrationDirection::Up;
        let cloned = direction;

        assert_eq!(cloned, MigrationDirection::Up);
    }

    #[test]
    fn test_migration_direction_debug() {
        let up = MigrationDirection::Up;
        let down = MigrationDirection::Down;

        assert_eq!(format!("{up:?}"), "Up");
        assert_eq!(format!("{down:?}"), "Down");
    }

    #[test]
    fn test_migration_direction_copy() {
        let original = MigrationDirection::Up;
        let copied: MigrationDirection = original;
        // original is still valid after being copied because MigrationDirection is Copy
        assert_eq!(original, MigrationDirection::Up);
        assert_eq!(copied, MigrationDirection::Up);
    }
}
