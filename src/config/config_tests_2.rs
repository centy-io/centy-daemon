use super::*;
use tokio::fs;

    #[test]
    fn test_custom_field_def_json_uses_camel_case() {
        let field = mdstore::CustomFieldDef {
            name: "test".to_string(), field_type: "string".to_string(), required: false,
            default_value: None, enum_values: vec![],
        };
        let json = serde_json::to_string(&field).expect("Should serialize");
        assert!(json.contains("\"type\""));
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"required\""));
        assert!(!json.contains("field_type"));
    }

    #[test]
    fn test_project_metadata_default() {
        let metadata = ProjectMetadata::default();
        assert!(metadata.title.is_none());
    }

    #[test]
    fn test_project_metadata_serialization() {
        let mut metadata = ProjectMetadata::default();
        metadata.title = Some("My Project".to_string());
        let json = serde_json::to_string(&metadata).expect("Should serialize");
        let deserialized: ProjectMetadata = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.title, Some("My Project".to_string()));
    }

    #[tokio::test]
    async fn test_read_config_nonexistent_returns_none() {
        use tempfile::tempdir;
        let temp_dir = tempdir().expect("Should create temp dir");
        let result = read_config(temp_dir.path()).await.expect("Should not error");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_read_write_config_roundtrip() {
        use tempfile::tempdir;
        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_dir = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_dir).await.expect("Should create .centy dir");
        let mut config = CentyConfig::default();
        config.version = Some("2.0.0".to_string());
        config.priority_levels = 4;
        write_config(temp_dir.path(), &config).await.expect("Should write");
        let read_config = read_config(temp_dir.path()).await.expect("Should read").expect("Config should exist");
        assert_eq!(read_config.version, Some("2.0.0".to_string()));
        assert_eq!(read_config.priority_levels, 4);
    }

    #[tokio::test]
    async fn test_read_project_metadata_nonexistent_returns_none() {
        use tempfile::tempdir;
        let temp_dir = tempdir().expect("Should create temp dir");
        let result = read_project_metadata(temp_dir.path()).await.expect("Should not error");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_read_write_project_metadata_roundtrip() {
        use tempfile::tempdir;
        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_dir = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_dir).await.expect("Should create .centy dir");
        let mut metadata = ProjectMetadata::default();
        metadata.title = Some("Test Project".to_string());
        write_project_metadata(temp_dir.path(), &metadata).await.expect("Should write");
        let read_metadata = read_project_metadata(temp_dir.path()).await.expect("Should read").expect("Metadata should exist");
        assert_eq!(read_metadata.title, Some("Test Project".to_string()));
    }

    #[tokio::test]
    async fn test_get_project_title_nonexistent() {
        use tempfile::tempdir;
        let temp_dir = tempdir().expect("Should create temp dir");
        let title = get_project_title(temp_dir.path()).await;
        assert!(title.is_none());
    }

    #[tokio::test]
    async fn test_set_and_get_project_title() {
        use tempfile::tempdir;
        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_dir = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_dir).await.expect("Should create .centy dir");
        set_project_title(temp_dir.path(), Some("My Awesome Project".to_string())).await.expect("Should set title");
        let title = get_project_title(temp_dir.path()).await;
        assert_eq!(title, Some("My Awesome Project".to_string()));
        set_project_title(temp_dir.path(), None).await.expect("Should clear title");
        let title = get_project_title(temp_dir.path()).await;
        assert!(title.is_none());
    }
