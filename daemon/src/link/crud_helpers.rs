use super::TargetType;
use crate::config::item_type_config::ItemTypeRegistry;
use crate::utils::get_centy_path;
use std::path::Path;

pub(super) async fn entity_exists(
    project_path: &Path,
    entity_id: &str,
    entity_type: &TargetType,
) -> bool {
    let folder = resolve_folder(project_path, entity_type).await;
    let centy_path = get_centy_path(project_path);
    let base_path = centy_path.join(&folder);
    if base_path.join(format!("{entity_id}.md")).exists() {
        return true;
    }
    base_path.join(entity_id).exists()
}

/// Resolve the actual storage folder for an entity type using the item type
/// registry. Falls back to the naive `folder_name()` (append "s") if the
/// registry lookup fails or the type is not found.
async fn resolve_folder(project_path: &Path, entity_type: &TargetType) -> String {
    if let Ok(registry) = ItemTypeRegistry::build(project_path).await {
        if let Some((folder, _)) = registry.resolve(entity_type.as_str()) {
            return folder.clone();
        }
    }
    entity_type.folder_name()
}
