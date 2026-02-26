use super::infer::{infer_organization_from_remote, OrgInferenceResult};
use std::path::Path;
/// Attempt to infer and auto-assign organization for a project if ungrouped.
pub async fn try_auto_assign_organization(
    project_path: &str,
    current_org_slug: Option<&str>,
) -> Option<OrgInferenceResult> {
    if let Some(slug) = current_org_slug {
        if !slug.is_empty() {
            return None;
        }
    }
    let path = Path::new(project_path);
    if !path.exists() {
        return None;
    }
    let inference = infer_organization_from_remote(path, current_org_slug).await;
    if !inference.has_mismatch {
        if let Some(ref slug) = inference.inferred_org_slug {
            let _ = super::super::organizations::set_project_organization(project_path, Some(slug))
                .await;
        }
    }
    Some(inference)
}
