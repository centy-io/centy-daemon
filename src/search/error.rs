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
