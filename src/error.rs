use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("ePub error: {0}")]
    EpubError(#[from] epub::error::Error),
    #[error("No chapters found")]
    NoChapters,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
