use super::types::ProjectRegistry;
use crate::utils::now_iso;
use tracing::info;

fn migrate_v1_to_v2(registry: &mut ProjectRegistry) {
    registry.schema_version = 2;
    registry.updated_at = now_iso();
    info!("Migrated registry from v1 to v2 (added organizations support)");
}

pub fn apply_migrations(registry: &mut ProjectRegistry) -> bool {
    let mut migrated = false;
    if registry.schema_version < 2 {
        migrate_v1_to_v2(registry);
        migrated = true;
    }
    migrated
}
