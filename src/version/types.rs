//! Version types for semantic versioning support.
//!
//! This module uses the standard `semver` crate for version handling,
//! re-exporting `semver::Version` as `SemVer` for API compatibility.

use thiserror::Error;

/// Re-export semver::Version as SemVer for API compatibility.
pub use semver::Version as SemVer;

/// Error types for version operations.
#[derive(Error, Debug)]
pub enum VersionError {
    #[error("Invalid version format: {0}")]
    InvalidFormat(String),

    #[error("Version not found in config")]
    NotFound,
}

impl From<semver::Error> for VersionError {
    fn from(err: semver::Error) -> Self {
        VersionError::InvalidFormat(err.to_string())
    }
}

/// Result of comparing project version against daemon version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionComparison {
    /// Project is at same version as daemon.
    Equal,
    /// Project is older than daemon (can upgrade).
    ProjectBehind,
    /// Project is newer than daemon (degraded mode).
    ProjectAhead,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse_valid() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_semver_parse_zero() {
        let v = SemVer::parse("0.0.0").unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_semver_parse_invalid_format() {
        assert!(SemVer::parse("1.2").is_err());
        assert!(SemVer::parse("1").is_err());
        assert!(SemVer::parse("1.2.3.4").is_err());
        assert!(SemVer::parse("").is_err());
    }

    #[test]
    fn test_semver_parse_invalid_number() {
        assert!(SemVer::parse("a.b.c").is_err());
        assert!(SemVer::parse("1.2.x").is_err());
    }

    #[test]
    fn test_semver_display() {
        let v = SemVer::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_semver_comparison() {
        let v1 = SemVer::new(1, 0, 0);
        let v2 = SemVer::new(2, 0, 0);
        let v3 = SemVer::new(1, 1, 0);
        let v4 = SemVer::new(1, 0, 1);
        let v5 = SemVer::new(1, 0, 0);

        assert!(v1 < v2);
        assert!(v1 < v3);
        assert!(v1 < v4);
        assert!(v1 == v5);
        assert!(v3 < v2);
        assert!(v4 < v3);
    }

    #[test]
    fn test_semver_ordering() {
        let mut versions = [
            SemVer::new(2, 0, 0),
            SemVer::new(0, 1, 0),
            SemVer::new(1, 0, 0),
            SemVer::new(1, 1, 0),
        ];
        versions.sort();

        assert_eq!(versions[0], SemVer::new(0, 1, 0));
        assert_eq!(versions[1], SemVer::new(1, 0, 0));
        assert_eq!(versions[2], SemVer::new(1, 1, 0));
        assert_eq!(versions[3], SemVer::new(2, 0, 0));
    }

    #[test]
    fn test_version_error_from_semver_error() {
        let err = SemVer::parse("invalid").unwrap_err();
        let version_err: VersionError = err.into();
        assert!(matches!(version_err, VersionError::InvalidFormat(_)));
    }
}
