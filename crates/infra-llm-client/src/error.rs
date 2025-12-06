//! Error types for LLM client operations.

use thiserror::Error;

/// Errors that can occur during LLM client operations.
#[derive(Error, Debug)]
pub enum LlmClientError {
    /// An error occurred while communicating with the LLM provider.
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// The API request was invalid.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// The API key or credentials are invalid or missing.
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// The API rate limit was exceeded.
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// The requested model was not found.
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// The request timeout was exceeded.
    #[error("Request timeout: {0}")]
    Timeout(String),

    /// An error occurred during serialization or deserialization.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// An underlying infrastructure error occurred.
    #[error("Infrastructure error: {0}")]
    InfraError(#[from] infra_errors::InfraError),

    /// A network or I/O error occurred.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// The response from the provider was invalid or unexpected.
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// The operation is not supported by this provider.
    #[error("Operation not supported: {0}")]
    Unsupported(String),

    /// An unknown or unexpected error occurred.
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// A specialized Result type for LLM client operations.
pub type Result<T> = std::result::Result<T, LlmClientError>;
