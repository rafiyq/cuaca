use thiserror::Error;

#[derive(Error, Debug)]
pub enum CuacaError {
    #[error("Location error: {0}")]
    Location(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Cache I/O error: {0}")]
    Cache(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Data error: {0}")]
    Data(String),

    #[error("Unexpected error: {0}")]
    Unknown(String),
}

// For warnings fetch failures, we may want a separate type or alias.
pub type WarningResult<T> = Result<T, CuacaError>;
