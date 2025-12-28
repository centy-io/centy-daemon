//! Platform detection logic for source control systems.

use crate::pr::remote::parse_remote_url;

/// Supported source control platforms
#[derive(Debug, Clone, PartialEq)]
pub enum SourceControlPlatform {
    GitHub,
    GitLab,
    Bitbucket,
    AzureDevOps,
    Gitea,
    /// Self-hosted git with unknown URL pattern
    Unknown { host: String },
}

impl SourceControlPlatform {
    /// Get human-readable name for the platform
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::GitHub => "GitHub",
            Self::GitLab => "GitLab",
            Self::Bitbucket => "Bitbucket",
            Self::AzureDevOps => "Azure DevOps",
            Self::Gitea => "Gitea",
            Self::Unknown { .. } => "Unknown",
        }
    }

    /// Check if this platform is supported for URL generation
    #[must_use]
    pub fn is_supported(&self) -> bool {
        !matches!(self, Self::Unknown { .. })
    }
}

/// Detect which source control platform is being used based on the remote URL.
///
/// Uses the existing `parse_remote_url` function to extract the host, then maps
/// known hosts to platforms.
///
/// # Arguments
/// * `remote_url` - The git remote URL (HTTPS or SSH format)
///
/// # Returns
/// * `Some(SourceControlPlatform)` if the platform could be detected
/// * `None` if the URL format is invalid
pub fn detect_platform(remote_url: &str) -> Option<SourceControlPlatform> {
    let parsed = parse_remote_url(remote_url)?;
    let host = parsed.host.to_lowercase();

    // Match known platform hosts
    let platform = if host.contains("github.com") {
        SourceControlPlatform::GitHub
    } else if host.contains("gitlab.com") {
        SourceControlPlatform::GitLab
    } else if host.contains("bitbucket.org") {
        SourceControlPlatform::Bitbucket
    } else if host.contains("dev.azure.com") || host.contains("visualstudio.com") {
        SourceControlPlatform::AzureDevOps
    } else if host.contains("gitea") {
        // Gitea instances often have "gitea" in the domain
        SourceControlPlatform::Gitea
    } else {
        // Unknown/self-hosted
        SourceControlPlatform::Unknown { host: parsed.host }
    };

    Some(platform)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_github() {
        assert_eq!(
            detect_platform("https://github.com/user/repo.git"),
            Some(SourceControlPlatform::GitHub)
        );
        assert_eq!(
            detect_platform("git@github.com:user/repo.git"),
            Some(SourceControlPlatform::GitHub)
        );
    }

    #[test]
    fn test_detect_gitlab() {
        assert_eq!(
            detect_platform("https://gitlab.com/user/repo.git"),
            Some(SourceControlPlatform::GitLab)
        );
        assert_eq!(
            detect_platform("git@gitlab.com:user/repo.git"),
            Some(SourceControlPlatform::GitLab)
        );
    }

    #[test]
    fn test_detect_bitbucket() {
        assert_eq!(
            detect_platform("https://bitbucket.org/team/repo.git"),
            Some(SourceControlPlatform::Bitbucket)
        );
        assert_eq!(
            detect_platform("git@bitbucket.org:team/repo.git"),
            Some(SourceControlPlatform::Bitbucket)
        );
    }

    #[test]
    fn test_detect_azure_devops() {
        assert_eq!(
            detect_platform("https://dev.azure.com/org/project/_git/repo"),
            Some(SourceControlPlatform::AzureDevOps)
        );
    }

    #[test]
    fn test_detect_gitea() {
        assert_eq!(
            detect_platform("https://gitea.example.com/user/repo.git"),
            Some(SourceControlPlatform::Gitea)
        );
    }

    #[test]
    fn test_detect_unknown_self_hosted() {
        if let Some(SourceControlPlatform::Unknown { host }) =
            detect_platform("https://git.company.com/team/repo.git")
        {
            assert_eq!(host, "git.company.com");
        } else {
            panic!("Expected Unknown platform");
        }
    }

    #[test]
    fn test_detect_invalid_url() {
        assert_eq!(detect_platform("not-a-url"), None);
    }

    #[test]
    fn test_platform_is_supported() {
        assert!(SourceControlPlatform::GitHub.is_supported());
        assert!(SourceControlPlatform::GitLab.is_supported());
        assert!(SourceControlPlatform::Bitbucket.is_supported());
        assert!(!SourceControlPlatform::Unknown {
            host: "example.com".to_string()
        }
        .is_supported());
    }
}
