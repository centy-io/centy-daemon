use super::TargetType;
use crate::utils::get_centy_path;
use std::path::Path;

pub(super) fn entity_exists(
    project_path: &Path,
    entity_id: &str,
    entity_type: &TargetType,
) -> bool {
    let centy_path = get_centy_path(project_path);
    let base_path = centy_path.join(entity_type.folder_name());
    if base_path.join(format!("{entity_id}.md")).exists() {
        return true;
    }
    base_path.join(entity_id).exists()
}
