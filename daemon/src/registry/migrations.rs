use super::types::ProjectRegistry;
use crate::utils::now_iso;
use tracing::info;

fn migrate_v1_to_v2(registry: &mut ProjectRegistry) {
    registry.schema_version = 2;
    registry.updated_at = now_iso();
    info!("Migrated registry from v1 to v2 (added organizations support)");
}

pub fn apply_migrations(registry: &mut ProjectRegistry) -> bool {
    if registry.schema_version < 2 {
        migrate_v1_to_v2(registry);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::registry::types::CURRENT_SCHEMA_VERSION;

    #[test]
    fn test_apply_migrations_v1_migrates_to_v2() {
        let mut registry = ProjectRegistry::default(); // schema_version = 0
        registry.schema_version = 1;
        let migrated = apply_migrations(&mut registry);
        assert!(migrated, "Should return true when migration occurred");
        assert_eq!(registry.schema_version, 2);
        assert!(!registry.updated_at.is_empty());
    }

    #[test]
    fn test_apply_migrations_v0_migrates_to_v2() {
        let mut registry = ProjectRegistry::default(); // schema_version = 0
        let migrated = apply_migrations(&mut registry);
        assert!(migrated, "Should return true when migration occurred");
        assert_eq!(registry.schema_version, 2);
    }

    #[test]
    fn test_apply_migrations_v2_no_migration() {
        let mut registry = ProjectRegistry::new(); // schema_version = CURRENT (2)
        assert_eq!(registry.schema_version, CURRENT_SCHEMA_VERSION);
        let migrated = apply_migrations(&mut registry);
        assert!(!migrated, "Should return false when no migration needed");
        assert_eq!(registry.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_apply_migrations_idempotent() {
        let mut registry = ProjectRegistry::default();
        registry.schema_version = 1;
        apply_migrations(&mut registry);
        // Applying again should be no-op
        let migrated_again = apply_migrations(&mut registry);
        assert!(!migrated_again);
        assert_eq!(registry.schema_version, 2);
    }
}
