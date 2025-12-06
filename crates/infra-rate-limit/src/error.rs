//! Error types for rate limiting operations.

use thiserror::Error;

/// Errors that can occur during rate limiting operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RateLimitError {
    /// Rate limit exceeded.
    #[error("rate limit exceeded, try again in {retry_after_ms}ms")]
    Exceeded {
        /// Milliseconds to wait before retrying.
        retry_after_ms: u64,
    },

    /// Configuration error.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl RateLimitError {
    /// Creates a new exceeded error with a retry-after duration.
    pub fn exceeded(retry_after_ms: u64) -> Self {
        Self::Exceeded { retry_after_ms }
    }

    /// Creates a new invalid configuration error.
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }

    /// Creates a new internal error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
