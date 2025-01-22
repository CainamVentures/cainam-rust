use thiserror::Error;

#[derive(Error, Debug)]
pub enum BirdeyeError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Cache error: {0}")]
    CacheError(String),
}

impl From<BirdeyeError> for rig_core::plugin::ActionError {
    fn from(error: BirdeyeError) -> Self {
        rig_core::plugin::ActionError::new(error.to_string())
    }
}
