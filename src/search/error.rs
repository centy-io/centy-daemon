use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Query parse error: {0}")]
    ParseError(String),

    #[error("Invalid operator '{0}' for field '{1}'")]
    InvalidOperator(String, String),

    #[error("Invalid value '{0}': {1}")]
    InvalidValue(String, String),

    #[error("Invalid date format '{0}': expected YYYY-MM-DD")]
    InvalidDateFormat(String),

    #[error("Invalid regex pattern '{0}': {1}")]
    InvalidRegex(String, String),

    #[error("Issue error: {0}")]
    IssueError(String),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_error_parse_error() {
        let err = SearchError::ParseError("unexpected token".to_string());
        assert_eq!(format!("{err}"), "Query parse error: unexpected token");
    }

    #[test]
    fn test_search_error_invalid_operator() {
        let err = SearchError::InvalidOperator(">>".to_string(), "status".to_string());
        let display = format!("{err}");
        assert!(display.contains("Invalid operator '>>'"));
        assert!(display.contains("'status'"));
    }

    #[test]
    fn test_search_error_invalid_value() {
        let err = SearchError::InvalidValue("abc".to_string(), "expected number".to_string());
        let display = format!("{err}");
        assert!(display.contains("Invalid value 'abc'"));
        assert!(display.contains("expected number"));
    }

    #[test]
    fn test_search_error_invalid_date_format() {
        let err = SearchError::InvalidDateFormat("not-a-date".to_string());
        let display = format!("{err}");
        assert!(display.contains("Invalid date format"));
        assert!(display.contains("YYYY-MM-DD"));
    }

    #[test]
    fn test_search_error_invalid_regex() {
        let err = SearchError::InvalidRegex("[unclosed".to_string(), "missing ]".to_string());
        let display = format!("{err}");
        assert!(display.contains("Invalid regex pattern"));
    }

    #[test]
    fn test_search_error_issue_error() {
        let err = SearchError::IssueError("issue listing failed".to_string());
        assert_eq!(format!("{err}"), "Issue error: issue listing failed");
    }

    #[test]
    fn test_search_error_registry_error() {
        let err = SearchError::RegistryError("registry unavailable".to_string());
        assert_eq!(format!("{err}"), "Registry error: registry unavailable");
    }

    #[test]
    fn test_search_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = SearchError::from(io_err);
        assert!(matches!(err, SearchError::IoError(_)));
    }
}
