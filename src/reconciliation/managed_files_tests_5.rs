use super::*;

    #[test]
    fn test_managed_file_template_issues_readme_content() {
        let files = get_managed_files();
        let readme = files
            .get("issues/README.md")
            .expect("Should have issues/README.md");

        let content = readme
            .content
            .as_ref()
            .expect("Issues README should have content");
        assert!(content.contains("Issues"));
        assert!(content.contains("AI Assistant Instructions"));
        assert!(content.contains("Reading Issues"));
        assert!(content.contains("Closing Issues"));
    }

    #[test]
    fn test_managed_file_template_templates_readme_content() {
        let files = get_managed_files();
        let readme = files
            .get("templates/README.md")
            .expect("Should have templates/README.md");

        let content = readme
            .content
            .as_ref()
            .expect("Templates README should have content");
        assert!(content.contains("Templates"));
        assert!(content.contains("Handlebars"));
        assert!(content.contains("{{title}}"));
        assert!(content.contains("{{description}}"));
    }

    #[test]
    fn test_managed_file_template_cspell_content() {
        let files = get_managed_files();
        let cspell = files.get("cspell.json").expect("Should have cspell.json");

        let content = cspell.content.as_ref().expect("cspell should have content");
        assert!(content.contains("centy"));
        assert!(content.contains("displayNumber"));
        assert!(content.contains("createdAt"));
        assert!(content.contains("allowedStates"));
    }
