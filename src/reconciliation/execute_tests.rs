use super::*;

    #[test]
    fn test_reconciliation_decisions_default() {
        let decisions = ReconciliationDecisions::default();
        assert!(decisions.restore.is_empty());
        assert!(decisions.reset.is_empty());
    }

    #[test]
    fn test_reconciliation_decisions_with_values() {
        let mut decisions = ReconciliationDecisions::default();
        decisions.restore.insert("README.md".to_string());
        decisions.reset.insert("config.json".to_string());
        assert!(decisions.restore.contains("README.md"));
        assert!(decisions.reset.contains("config.json"));
        assert_eq!(decisions.restore.len(), 1);
        assert_eq!(decisions.reset.len(), 1);
    }

    #[test]
    fn test_reconciliation_decisions_clone() {
        let mut decisions = ReconciliationDecisions::default();
        decisions.restore.insert("test.md".to_string());
        let cloned = decisions.clone();
        assert!(cloned.restore.contains("test.md"));
    }

    #[test]
    fn test_reconciliation_decisions_debug() {
        let decisions = ReconciliationDecisions::default();
        let debug_str = format!("{decisions:?}");
        assert!(debug_str.contains("ReconciliationDecisions"));
    }

    #[test]
    fn test_reconciliation_result_default() {
        let result = ReconciliationResult::default();
        assert!(result.created.is_empty());
        assert!(result.restored.is_empty());
        assert!(result.reset.is_empty());
        assert!(result.skipped.is_empty());
    }

    #[test]
    fn test_reconciliation_result_with_values() {
        let mut result = ReconciliationResult::default();
        result.created.push("README.md".to_string());
        result.restored.push("config.json".to_string());
        result.reset.push("issues/README.md".to_string());
        result.skipped.push("custom.md".to_string());
        assert_eq!(result.created.len(), 1);
        assert_eq!(result.restored.len(), 1);
        assert_eq!(result.reset.len(), 1);
        assert_eq!(result.skipped.len(), 1);
    }

    #[test]
    fn test_reconciliation_result_clone() {
        let mut result = ReconciliationResult::default();
        result.created.push("test.md".to_string());
        let cloned = result.clone();
        assert_eq!(cloned.created, vec!["test.md".to_string()]);
    }

    #[test]
    fn test_reconciliation_result_debug() {
        let result = ReconciliationResult::default();
        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("ReconciliationResult"));
    }

    #[test]
    fn test_execute_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let execute_err = ExecuteError::IoError(io_err);
        let display = format!("{execute_err}");
        assert!(display.contains("IO error"));
    }

    #[test]
    fn test_execute_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let execute_err = ExecuteError::from(io_err);
        assert!(matches!(execute_err, ExecuteError::IoError(_)));
    }
