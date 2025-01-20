use thiserror::Error;

#[derive(Debug, Error)]
pub enum BirdeyeError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("API error: {status_code} - {message}")]
    ApiError {
        status_code: u16,
        message: String,
    },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Invalid address format: {0}")]
    InvalidAddress(String),

    #[error("Invalid chain: {0}")]
    InvalidChain(String),

    #[error("Invalid time interval: {0}")]
    InvalidTimeInterval(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Token not found: {0}")]
    TokenNotFound(String),

    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl BirdeyeError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            BirdeyeError::RequestError(_) | BirdeyeError::RateLimitExceeded
        )
    }

    pub fn from_status_and_message(status: u16, message: impl Into<String>) -> Self {
        match status {
            429 => BirdeyeError::RateLimitExceeded,
            401 => BirdeyeError::InvalidApiKey,
            404 => {
                let msg = message.into();
                if msg.contains("token") {
                    BirdeyeError::TokenNotFound(msg)
                } else if msg.contains("wallet") {
                    BirdeyeError::WalletNotFound(msg)
                } else {
                    BirdeyeError::ApiError {
                        status_code: status,
                        message: msg,
                    }
                }
            }
            _ => BirdeyeError::ApiError {
                status_code: status,
                message: message.into(),
            },
        }
    }
}

pub type BirdeyeResult<T> = Result<T, BirdeyeError>; 