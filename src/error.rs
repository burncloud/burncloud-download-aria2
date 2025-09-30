use thiserror::Error;

#[derive(Error, Debug)]
pub enum Aria2Error {
    #[error("JSON-RPC error: code={0}, message={1}")]
    RpcError(i32, String),

    #[error("HTTP transport error: {0}")]
    TransportError(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Aria2 daemon unavailable: {0}")]
    DaemonUnavailable(String),

    #[error("Task not found in aria2: {0}")]
    TaskNotFound(String),

    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),

    #[error("Invalid file path: {0}")]
    InvalidPath(String),

    #[error("Unsupported download type: {0}")]
    UnsupportedType(String),

    #[error("State mapping error: {0}")]
    StateMappingError(String),

    #[error("General error: {0}")]
    General(String),
}