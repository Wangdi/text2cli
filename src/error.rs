use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Agent execution failed: {0}")]
    AgentExecution(String),

    #[error("No command returned from agent")]
    NoCommandReturned,

    #[error("IO error: {0}")]
    Io(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("JSON error: {0}")]
    Json(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
