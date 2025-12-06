//! Error types for cache operations.

use thiserror::Error;

/// Errors that can occur during cache operations.
#[derive(Debug, Error)]
pub enum CacheError {
    /// Key not found in cache.
    #[error("Cache key not found: {0}")]
    KeyNotFound(String),

    /// Failed to serialize value.
    #[error("Failed to serialize cache value: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Failed to deserialize value.
    #[error("Failed to deserialize cache value: {0}")]
    DeserializationError(String),

    /// Cache is full and cannot accept new entries.
    #[error("Cache is full (max size: {0})")]
    CacheFull(usize),

    /// Entry has expired.
    #[error("Cache entry has expired")]
    EntryExpired,

    /// Invalid cache configuration.
    #[error("Invalid cache configuration: {0}")]
    InvalidConfig(String),

    /// Network error for distributed caches.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Generic cache error.
    #[error("Cache error: {0}")]
    Other(String),
}

/// Result type for cache operations.
pub type CacheResult<T> = Result<T, CacheError>;
