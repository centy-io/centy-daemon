use super::*;

    #[test]
    fn test_merge_json_content_unions_words() {
        let existing = r#"{
  "version": "0.1",
  "language": "en",
  "words": ["alpha", "centy", "custom"],
  "ignorePaths": [".centy-manifest.json"]
}"#;
        let template = r#"{
  "version": "0.2",
  "language": "en",
  "words": ["centy", "displayNumber", "createdAt"],
  "ignorePaths": [".centy-manifest.json"]
}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let words: Vec<&str> = parsed["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            words,
            vec!["alpha", "centy", "createdAt", "custom", "displayNumber"]
        );
    }

    #[test]
    fn test_merge_json_content_uses_template_version() {
        let existing = r#"{"version": "0.1", "language": "fr", "words": ["custom"]}"#;
        let template = r#"{"version": "0.2", "language": "en", "words": ["centy"]}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["version"], "0.2");
        assert_eq!(parsed["language"], "en");
    }

    #[test]
    fn test_merge_json_content_preserves_user_keys() {
        let existing =
            r#"{"version": "0.1", "language": "en", "words": [], "flagWords": ["forbidden"]}"#;
        let template = r#"{"version": "0.2", "language": "en", "words": ["centy"]}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["flagWords"][0], "forbidden");
    }

    #[test]
    fn test_merge_json_content_unions_ignore_paths() {
        let existing = r#"{
  "version": "0.2",
  "language": "en",
  "words": [],
  "ignorePaths": [".centy-manifest.json", "custom-path/"]
}"#;
        let template = r#"{
  "version": "0.2",
  "language": "en",
  "words": [],
  "ignorePaths": [".centy-manifest.json", "node_modules/"]
}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let paths: Vec<&str> = parsed["ignorePaths"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            paths,
            vec![".centy-manifest.json", "custom-path/", "node_modules/"]
        );
    }
