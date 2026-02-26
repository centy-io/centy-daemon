use super::*;

    #[test]
    fn test_workspace_config_serialization_with_value() {
        let ws = WorkspaceConfig { update_status_on_open: Some(true) };
        let json = serde_json::to_string(&ws).expect("Should serialize");
        assert!(json.contains("updateStatusOnOpen"));
        let deserialized: WorkspaceConfig = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.update_status_on_open, Some(true));
    }

    #[test]
    fn test_centy_config_workspace_roundtrip() {
        let mut config = CentyConfig::default();
        config.workspace.update_status_on_open = Some(false);
        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: CentyConfig = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.workspace.update_status_on_open, Some(false));
    }
