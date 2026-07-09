use thiserror::Error;

pub type Result<T> = std::result::Result<T, BolootError>;

#[derive(Debug, Error)]
pub enum BolootError {
    #[error("invalid date: {0}")]
    InvalidDate(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("location not found: {0}")]
    LocationNotFound(String),

    #[error("holiday data error: {0}")]
    Holiday(String),

    #[error("prayer calculation error: {0}")]
    Prayer(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}
