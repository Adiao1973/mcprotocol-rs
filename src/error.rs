use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("JSON-RPC error: {code} - {message}")]
    JsonRpc { code: i32, message: String },

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
