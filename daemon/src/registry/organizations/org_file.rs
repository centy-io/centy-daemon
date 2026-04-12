use super::errors::OrganizationError;
use crate::registry::types::ProjectOrganization;
use crate::utils::get_centy_path;
use std::path::Path;
use tokio::fs;

/// Read the .centy/organization.json file from a project
pub async fn read_project_org_file(
    project_path: &Path,
) -> Result<Option<ProjectOrganization>, OrganizationError> {
    let centy_path = get_centy_path(project_path);
    let org_file_path = centy_path.join("organization.json");

    if !org_file_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&org_file_path).await?;
    let org: ProjectOrganization = serde_json::from_str(&content)?;

    Ok(Some(org))
}

/// Write the .centy/organization.json file
pub(super) async fn write_project_org_file(
    path: &Path,
    org: &ProjectOrganization,
) -> Result<(), OrganizationError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let content = serde_json::to_string_pretty(org)?;
    fs::write(path, content).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use tempfile::TempDir;

    fn make_project_org() -> ProjectOrganization {
        ProjectOrganization {
            slug: "test-slug".to_string(),
            name: "Test Org".to_string(),
            description: Some("A test".to_string()),
        }
    }

    #[tokio::test]
    async fn test_read_project_org_file_returns_none_when_missing() {
        let dir = TempDir::new().expect("tmp");
        let result = read_project_org_file(dir.path()).await.expect("should ok");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_write_and_read_project_org_file() {
        let dir = TempDir::new().expect("tmp");
        let project_path = dir.path();
        let centy_dir = project_path.join(".centy");
        tokio::fs::create_dir_all(&centy_dir)
            .await
            .expect("create .centy");

        let org_file_path = centy_dir.join("organization.json");
        let org = make_project_org();
        write_project_org_file(&org_file_path, &org)
            .await
            .expect("write");

        let result = read_project_org_file(project_path)
            .await
            .expect("read")
            .expect("should be Some");

        assert_eq!(result.slug, "test-slug");
        assert_eq!(result.name, "Test Org");
        assert_eq!(result.description, Some("A test".to_string()));
    }

    #[tokio::test]
    async fn test_read_project_org_file_invalid_json() {
        let dir = TempDir::new().expect("tmp");
        let centy_dir = dir.path().join(".centy");
        tokio::fs::create_dir_all(&centy_dir)
            .await
            .expect("create .centy");

        let org_file = centy_dir.join("organization.json");
        tokio::fs::write(&org_file, b"{ bad json }")
            .await
            .expect("write bad");

        let result = read_project_org_file(dir.path()).await;
        assert!(result.is_err(), "should error on invalid JSON");
    }

    #[tokio::test]
    async fn test_write_project_org_file_creates_parent_dir() {
        let dir = TempDir::new().expect("tmp");
        // Write to a deeply nested path that doesn't exist yet
        let nested = dir.path().join("deep").join("nested").join("org.json");
        let org = make_project_org();
        write_project_org_file(&nested, &org)
            .await
            .expect("should create parents and write");
        assert!(nested.exists());
    }
}
