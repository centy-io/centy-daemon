use super::*;

    #[test]
    fn test_merge_json_content_sorted_output() {
        let existing = r#"{"version": "0.2", "language": "en", "words": ["zebra", "apple"]}"#;
        let template = r#"{"version": "0.2", "language": "en", "words": ["mango", "centy"]}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let words: Vec<&str> = parsed["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(words, vec!["apple", "centy", "mango", "zebra"]);
    }

    #[test]
    fn test_merge_json_content_deduplicates() {
        let existing =
            r#"{"version": "0.2", "language": "en", "words": ["centy", "centy", "alpha"]}"#;
        let template = r#"{"version": "0.2", "language": "en", "words": ["centy", "alpha"]}"#;

        let result = merge_json_content(existing, template).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let words: Vec<&str> = parsed["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(words, vec!["alpha", "centy"]);
    }

    #[test]
    fn test_merge_json_content_trailing_newline() {
        let existing = r#"{"version": "0.1", "language": "en", "words": []}"#;
        let template = r#"{"version": "0.2", "language": "en", "words": []}"#;

        let result = merge_json_content(existing, template).unwrap();
        assert!(result.ends_with('\n'));
    }
