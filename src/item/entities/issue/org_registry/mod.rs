//! Organization-level issue display number registry.
//!
//! Manages central tracking of display numbers for org-level issues.
//! Stored at ~/.centy/org-issues-registry.json
mod types;
pub use types::{OrgIssueRegistry, OrgIssueRegistryError};

use crate::utils::now_iso;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::fs;
use tokio::sync::Mutex;

static ORG_ISSUE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn get_lock() -> &'static Mutex<()> {
    ORG_ISSUE_LOCK.get_or_init(|| Mutex::new(()))
}

fn get_centy_config_dir() -> Result<PathBuf, OrgIssueRegistryError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_e| OrgIssueRegistryError::HomeDirNotFound)?;
    Ok(PathBuf::from(home).join(".centy"))
}

fn get_registry_path() -> Result<PathBuf, OrgIssueRegistryError> {
    Ok(get_centy_config_dir()?.join("org-issues-registry.json"))
}

pub async fn read_org_issue_registry() -> Result<OrgIssueRegistry, OrgIssueRegistryError> {
    let path = get_registry_path()?;
    if !path.exists() {
        return Ok(OrgIssueRegistry::new());
    }
    let content = fs::read_to_string(&path).await?;
    Ok(serde_json::from_str(&content)?)
}

async fn write_registry_unlocked(registry: &OrgIssueRegistry) -> Result<(), OrgIssueRegistryError> {
    let path = get_registry_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let temp_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(registry)?;
    fs::write(&temp_path, &content).await?;
    fs::rename(&temp_path, &path).await?;
    Ok(())
}

pub async fn get_next_org_display_number(org_slug: &str) -> Result<u32, OrgIssueRegistryError> {
    let _guard = get_lock().lock().await;
    let mut registry = read_org_issue_registry().await?;
    let next = *registry.next_display_number.get(org_slug).unwrap_or(&1);
    registry
        .next_display_number
        .insert(org_slug.to_string(), next.saturating_add(1));
    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;
    Ok(next)
}

pub async fn get_current_org_display_number(org_slug: &str) -> Result<u32, OrgIssueRegistryError> {
    let registry = read_org_issue_registry().await?;
    let next = *registry.next_display_number.get(org_slug).unwrap_or(&1);
    Ok(next.saturating_sub(1))
}

#[cfg(test)]
#[path = "../org_registry_tests.rs"]
mod tests;
