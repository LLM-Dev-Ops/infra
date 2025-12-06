//! Core rate limiter trait and result types.

use crate::error::RateLimitError;
use async_trait::async_trait;
use std::time::Duration;

/// Result of a rate limit check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitResult {
    /// Request is allowed.
    Allowed,

    /// Request is denied.
    Denied {
        /// Duration to wait before retrying.
        wait_time: Duration,
    },
}

impl RateLimitResult {
    /// Returns true if the request is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed)
    }

    /// Returns true if the request is denied.
    pub fn is_denied(&self) -> bool {
        matches!(self, RateLimitResult::Denied { .. })
    }

    /// Returns the wait time if denied.
    pub fn wait_time(&self) -> Option<Duration> {
        match self {
            RateLimitResult::Allowed => None,
            RateLimitResult::Denied { wait_time } => Some(*wait_time),
        }
    }
}

/// Trait for rate limiting strategies.
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Attempts to acquire a permit, waiting if necessary.
    ///
    /// This method will wait until a permit becomes available.
    async fn acquire(&self) -> Result<(), RateLimitError>;

    /// Attempts to acquire a permit without waiting.
    ///
    /// Returns immediately with the result.
    async fn try_acquire(&self) -> RateLimitResult;

    /// Returns the number of available permits.
    async fn available(&self) -> u64;

    /// Resets the rate limiter state.
    async fn reset(&self);
}
