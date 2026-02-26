use super::*;

#[test]
fn test_org_sync_result_serialization() {
    let result = OrgSyncResult {
        project_path: "/path/to/project".to_string(),
        success: true,
        error: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"project_path\""));
    assert!(json.contains("\"success\":true"));
    // error should be omitted when None
    assert!(!json.contains("\"error\""));
}

#[test]
fn test_org_sync_result_with_error() {
    let result = OrgSyncResult {
        project_path: "/path/to/project".to_string(),
        success: false,
        error: Some("Failed to sync".to_string()),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"success\":false"));
    assert!(json.contains("\"error\":\"Failed to sync\""));
}
