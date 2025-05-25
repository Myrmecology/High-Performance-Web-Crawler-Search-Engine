use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    
    #[error("HTML parsing error: {0}")]
    HtmlParseError(String),
    
    #[error("Storage error: {0}")]
    StorageError(#[from] sqlx::Error),
    
    #[error("Index error: {0}")]
    IndexError(#[from] tantivy::TantivyError),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Rate limit exceeded for domain: {0}")]
    RateLimitError(String),
    
    #[error("Robots.txt forbids crawling: {0}")]
    RobotsForbidden(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Timeout occurred")]
    Timeout,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;