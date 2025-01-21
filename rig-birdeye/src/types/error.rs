use thiserror::Error;

#[derive(Debug, Error)]
pub enum BirdeyeError {
    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Token not found: {0}")]
    TokenNotFound(String),

    #[error("Invalid wallet address: {0}")]
    InvalidWalletAddress(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("API error: {status} - {message}")]
    ApiError {
        status: u16,
        message: String,
    },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<BirdeyeError> for rig_core::plugin::ActionError {
    fn from(error: BirdeyeError) -> Self {
        rig_core::plugin::ActionError::new(error.to_string())
    }
}
