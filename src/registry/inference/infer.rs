use super::super::organizations::{create_organization, get_organization, slugify};
use crate::common::git::{get_remote_origin_url, is_git_repository};
use crate::common::remote::parse_remote_url;
use std::path::Path;
/// Result of organization inference from git remote
#[derive(Debug, Clone, Default)]
pub struct OrgInferenceResult {
    pub inferred_org_slug: Option<String>,
    pub inferred_org_name: Option<String>,
    pub org_created: bool,
    pub existing_org_slug: Option<String>,
    pub has_mismatch: bool,
    pub message: Option<String>,
}
/// Infer organization from git remote URL.
#[allow(unknown_lints, max_lines_per_function, clippy::too_many_lines)]
pub async fn infer_organization_from_remote(
    project_path: &Path,
    existing_org_slug: Option<&str>,
) -> OrgInferenceResult {
    let mut result = OrgInferenceResult {
        existing_org_slug: existing_org_slug.map(String::from),
        ..Default::default()
    };
    if !is_git_repository(project_path) {
        result.message = Some("Not a git repository".to_string());
        return result;
    }
    let remote_url = match get_remote_origin_url(project_path) {
        Ok(url) => url,
        Err(_) => { result.message = Some("No origin remote found".to_string()); return result; }
    };
    let parsed = match parse_remote_url(&remote_url) {
        Some(p) => p,
        None => {
            result.message = Some(format!("Could not parse remote URL: {remote_url}"));
            return result;
        }
    };
    let inferred_slug = slugify(&parsed.org);
    result.inferred_org_slug = Some(inferred_slug.clone());
    result.inferred_org_name = Some(parsed.org.clone());
    if let Some(existing) = existing_org_slug {
        if !existing.is_empty() && existing != inferred_slug {
            result.has_mismatch = true;
            result.message = Some(format!(
                "Project is assigned to '{existing}' but git remote suggests '{inferred_slug}'. \
                 Use `centy org set {inferred_slug}` to update."
            ));
            return result;
        }
    }
    match get_organization(&inferred_slug).await {
        Ok(Some(_)) => { result.message = Some(format!("Using existing organization: {inferred_slug}")); }
        Ok(None) => {
            match create_organization(Some(&inferred_slug), &parsed.org, None).await {
                Ok(_) => {
                    result.org_created = true;
                    result.message = Some(format!("Created organization: {inferred_slug}"));
                }
                Err(e) => {
                    result.message = Some(format!("Failed to create organization: {e}"));
                    result.inferred_org_slug = None;
                    result.inferred_org_name = None;
                }
            }
        }
        Err(e) => {
            result.message = Some(format!("Error checking organization: {e}"));
            result.inferred_org_slug = None;
            result.inferred_org_name = None;
        }
    }
    result
}
