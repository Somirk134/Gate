use thiserror::Error;

#[derive(Debug, Error)]
pub enum SharedError {
    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

impl From<serde_json::Error> for SharedError {
    fn from(err: serde_json::Error) -> Self {
        SharedError::Serialization(err.to_string())
    }
}
