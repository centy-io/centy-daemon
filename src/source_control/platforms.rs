//! Platform-specific URL builders for source control systems.

use std::path::Path;

use crate::pr::git::{detect_current_branch, get_default_branch, get_remote_origin_url};
use crate::pr::remote::parse_remote_url;

use super::detection::{detect_platform, SourceControlPlatform};
use super::SourceControlError;

/// A generated source control URL with metadata
#[derive(Debug, Clone, PartialEq)]
pub struct SourceControlUrl {
    /// The detected platform
    pub platform: SourceControlPlatform,
    /// The full URL to open in a browser
    pub url: String,
}

/// Build a URL to view a folder in the source control web UI.
///
/// This function:
/// 1. Gets the git remote origin URL
/// 2. Detects which platform is being used
/// 3. Determines the current branch (or falls back to default)
/// 4. Generates a platform-specific URL
///
/// # Arguments
/// * `project_path` - Path to the git repository root
/// * `relative_path` - Relative path from repo root to the folder (e.g., ".centy/issues/123")
///
/// # Returns
/// * `Ok(SourceControlUrl)` with the generated URL
/// * `Err(SourceControlError)` if git repo not found, no remote, or unsupported platform
///
/// # Examples
/// ```no_run
/// use std::path::Path;
/// use centy_daemon::source_control::build_folder_url;
///
/// let url = build_folder_url(
///     Path::new("/path/to/repo"),
///     ".centy/issues/abc-123"
/// ).unwrap();
/// println!("Open: {}", url.url);
/// ```
pub fn build_folder_url(
    project_path: &Path,
    relative_path: &str,
) -> Result<SourceControlUrl, SourceControlError> {
    // Get the remote origin URL
    let remote_url = get_remote_origin_url(project_path)?;

    // Parse the remote URL to extract org/repo
    let parsed_remote = parse_remote_url(&remote_url)
        .ok_or_else(|| SourceControlError::InvalidRemoteUrl(remote_url.clone()))?;

    // Detect the platform
    let platform = detect_platform(&remote_url)
        .ok_or_else(|| SourceControlError::InvalidRemoteUrl(remote_url.clone()))?;

    // Check if platform is supported
    if !platform.is_supported() {
        return Err(SourceControlError::UnsupportedPlatform(
            platform.name().to_string(),
        ));
    }

    // Get current branch, fallback to default
    let branch = detect_current_branch(project_path)
        .unwrap_or_else(|_| get_default_branch(project_path));

    // Build platform-specific URL
    let url = build_platform_url(&platform, &parsed_remote, &branch, relative_path);

    Ok(SourceControlUrl { platform, url })
}

/// Build a platform-specific URL
fn build_platform_url(
    platform: &SourceControlPlatform,
    parsed_remote: &crate::pr::remote::ParsedRemote,
    branch: &str,
    relative_path: &str,
) -> String {
    let org = &parsed_remote.org;
    let repo = &parsed_remote.repo;
    let host = &parsed_remote.host;

    // Normalize path: remove leading/trailing slashes
    let path = relative_path.trim_start_matches('/').trim_end_matches('/');

    match platform {
        SourceControlPlatform::GitHub => {
            // GitHub: https://github.com/{org}/{repo}/tree/{branch}/{path}
            format!("https://{host}/{org}/{repo}/tree/{branch}/{path}")
        }
        SourceControlPlatform::GitLab => {
            // GitLab: https://gitlab.com/{org}/{repo}/-/tree/{branch}/{path}
            format!("https://{host}/{org}/{repo}/-/tree/{branch}/{path}")
        }
        SourceControlPlatform::Bitbucket => {
            // Bitbucket: https://bitbucket.org/{org}/{repo}/src/{branch}/{path}
            format!("https://{host}/{org}/{repo}/src/{branch}/{path}")
        }
        SourceControlPlatform::AzureDevOps => {
            // Azure DevOps: https://dev.azure.com/{org}/_git/{repo}?path=/{path}&version=GB{branch}
            format!(
                "https://{host}/{org}/_git/{repo}?path=/{path}&version=GB{branch}"
            )
        }
        SourceControlPlatform::Gitea => {
            // Gitea: https://{host}/{org}/{repo}/src/branch/{branch}/{path}
            format!("https://{host}/{org}/{repo}/src/branch/{branch}/{path}")
        }
        SourceControlPlatform::Unknown { .. } => {
            // Should not reach here due to supported check, but fallback to empty string
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a ParsedRemote
    fn make_parsed_remote(host: &str, org: &str, repo: &str) -> crate::pr::remote::ParsedRemote {
        crate::pr::remote::ParsedRemote {
            host: host.to_string(),
            org: org.to_string(),
            repo: repo.to_string(),
        }
    }

    #[test]
    fn test_github_url() {
        let platform = SourceControlPlatform::GitHub;
        let parsed = make_parsed_remote("github.com", "centy-io", "centy-daemon");
        let url = build_platform_url(&platform, &parsed, "main", ".centy/issues/123");
        assert_eq!(
            url,
            "https://github.com/centy-io/centy-daemon/tree/main/.centy/issues/123"
        );
    }

    #[test]
    fn test_gitlab_url() {
        let platform = SourceControlPlatform::GitLab;
        let parsed = make_parsed_remote("gitlab.com", "myorg", "myrepo");
        let url = build_platform_url(&platform, &parsed, "develop", ".centy/docs");
        assert_eq!(
            url,
            "https://gitlab.com/myorg/myrepo/-/tree/develop/.centy/docs"
        );
    }

    #[test]
    fn test_bitbucket_url() {
        let platform = SourceControlPlatform::Bitbucket;
        let parsed = make_parsed_remote("bitbucket.org", "team", "project");
        let url = build_platform_url(&platform, &parsed, "master", ".centy/prs/abc");
        assert_eq!(
            url,
            "https://bitbucket.org/team/project/src/master/.centy/prs/abc"
        );
    }

    #[test]
    fn test_azure_devops_url() {
        let platform = SourceControlPlatform::AzureDevOps;
        let parsed = make_parsed_remote("dev.azure.com", "myorg", "myrepo");
        let url = build_platform_url(&platform, &parsed, "main", "src/app");
        assert_eq!(
            url,
            "https://dev.azure.com/myorg/_git/myrepo?path=/src/app&version=GBmain"
        );
    }

    #[test]
    fn test_gitea_url() {
        let platform = SourceControlPlatform::Gitea;
        let parsed = make_parsed_remote("gitea.example.com", "user", "project");
        let url = build_platform_url(&platform, &parsed, "main", "docs");
        assert_eq!(
            url,
            "https://gitea.example.com/user/project/src/branch/main/docs"
        );
    }

    #[test]
    fn test_path_normalization() {
        let platform = SourceControlPlatform::GitHub;
        let parsed = make_parsed_remote("github.com", "org", "repo");

        // Test with leading/trailing slashes
        let url1 = build_platform_url(&platform, &parsed, "main", "/path/to/folder/");
        let url2 = build_platform_url(&platform, &parsed, "main", "path/to/folder");
        assert_eq!(url1, url2);
    }
}
