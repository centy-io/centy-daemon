use super::*;

    #[test]
    fn test_reconciliation_plan_default() {
        let plan = ReconciliationPlan::default();
        assert!(plan.to_create.is_empty());
        assert!(plan.to_restore.is_empty());
        assert!(plan.to_reset.is_empty());
        assert!(plan.up_to_date.is_empty());
        assert!(plan.user_files.is_empty());
    }

    #[test]
    fn test_reconciliation_plan_needs_decisions_false_when_empty() {
        let plan = ReconciliationPlan::default();
        assert!(!plan.needs_decisions());
    }

    #[test]
    fn test_reconciliation_plan_needs_decisions_true_when_restore_not_empty() {
        let mut plan = ReconciliationPlan::default();
        plan.to_restore.push(FileInfo { path: "test.md".to_string(), file_type: ManagedFileType::File, hash: "abc123".to_string(), content_preview: None });
        assert!(plan.needs_decisions());
    }

    #[test]
    fn test_reconciliation_plan_needs_decisions_true_when_reset_not_empty() {
        let mut plan = ReconciliationPlan::default();
        plan.to_reset.push(FileInfo { path: "test.md".to_string(), file_type: ManagedFileType::File, hash: "abc123".to_string(), content_preview: None });
        assert!(plan.needs_decisions());
    }

    #[test]
    fn test_file_info_initialization() {
        let file_info = FileInfo { path: "README.md".to_string(), file_type: ManagedFileType::File, hash: "abc123".to_string(), content_preview: Some("# Title".to_string()) };
        assert_eq!(file_info.path, "README.md");
        assert_eq!(file_info.file_type, ManagedFileType::File);
        assert_eq!(file_info.hash, "abc123");
        assert_eq!(file_info.content_preview, Some("# Title".to_string()));
    }

    #[test]
    fn test_file_info_clone() {
        let file_info = FileInfo { path: "test.md".to_string(), file_type: ManagedFileType::Directory, hash: String::new(), content_preview: None };
        let cloned = file_info.clone();
        assert_eq!(cloned.path, "test.md");
        assert_eq!(cloned.file_type, ManagedFileType::Directory);
    }

    #[test]
    fn test_file_info_debug() {
        let file_info = FileInfo { path: "debug.md".to_string(), file_type: ManagedFileType::File, hash: "hash123".to_string(), content_preview: Some("preview".to_string()) };
        let debug_str = format!("{file_info:?}");
        assert!(debug_str.contains("FileInfo"));
        assert!(debug_str.contains("debug.md"));
        assert!(debug_str.contains("hash123"));
    }

    #[test]
    fn test_plan_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let plan_err = PlanError::IoError(io_err);
        let display = format!("{plan_err}");
        assert!(display.contains("IO error"));
    }
