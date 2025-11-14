use thiserror::Error;

/// Main error type for LazyJira
#[derive(Error, Debug)]
pub enum LazyJiraError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Validation error: {0}")]
    #[allow(dead_code)] // Will be used for validation errors
    Validation(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Internal error: {0}")]
    #[allow(dead_code)] // Will be used for internal errors
    Internal(String),
}

/// Result type alias for LazyJira operations
pub type Result<T> = std::result::Result<T, LazyJiraError>;
