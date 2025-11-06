use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Memory not found: {0}")]
    NotFound(String),

    #[error("Invalid ULID: {0}")]
    InvalidId(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, MemoryError>;
