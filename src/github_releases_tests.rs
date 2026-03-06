use super::*;

#[test]
fn test_is_update_available() {
    assert!(is_update_available("0.8.0", "0.8.1"));
    assert!(is_update_available("0.8.1", "0.9.0"));
    assert!(is_update_available("0.8.1", "1.0.0"));
    assert!(!is_update_available("0.8.1", "0.8.1"));
    assert!(!is_update_available("0.8.1", "0.8.0"));
    assert!(!is_update_available("0.8.1", "0.7.0"));
}

#[test]
fn test_latest_stable_version() {
    let releases = vec![
        GitHubRelease {
            tag_name: "v1.0.0-rc1".to_string(),
            name: Some("Release Candidate".to_string()),
            published_at: Some("2026-01-01T00:00:00Z".to_string()),
            prerelease: true,
            html_url: "https://example.com".to_string(),
        },
        GitHubRelease {
            tag_name: "v0.9.0".to_string(),
            name: Some("Latest Stable".to_string()),
            published_at: Some("2025-12-01T00:00:00Z".to_string()),
            prerelease: false,
            html_url: "https://example.com".to_string(),
        },
    ];
    assert_eq!(latest_stable_version(&releases), Some("0.9.0".to_string()));
}

#[test]
fn test_latest_stable_version_empty() {
    let releases: Vec<GitHubRelease> = vec![];
    assert_eq!(latest_stable_version(&releases), None);
}

#[test]
fn test_latest_stable_version_only_prerelease() {
    let releases = vec![GitHubRelease {
        tag_name: "v1.0.0-rc1".to_string(),
        name: Some("RC".to_string()),
        published_at: None,
        prerelease: true,
        html_url: "https://example.com".to_string(),
    }];
    assert_eq!(latest_stable_version(&releases), None);
}
