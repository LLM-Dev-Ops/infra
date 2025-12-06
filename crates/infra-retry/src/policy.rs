//! Retry policy trait and decision types.

use std::time::Duration;

/// Decision made by a retry policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDecision {
    /// Retry the operation after the specified delay.
    Retry(Duration),
    /// Stop retrying and fail.
    Stop,
}

/// Trait for defining retry strategies.
///
/// Implementors of this trait define how and when operations should be retried,
/// including the delay between attempts and the maximum number of attempts.
pub trait RetryPolicy: Send + Sync {
    /// Determines whether a failed operation should be retried.
    ///
    /// # Arguments
    ///
    /// * `attempt` - The current attempt number (0-indexed).
    /// * `error` - The error that occurred during the last attempt.
    ///
    /// # Returns
    ///
    /// A `RetryDecision` indicating whether to retry and with what delay.
    fn should_retry(&self, attempt: u32, error: &dyn std::error::Error) -> RetryDecision;

    /// Returns the delay to wait before the next retry attempt.
    ///
    /// # Arguments
    ///
    /// * `attempt` - The current attempt number (0-indexed).
    ///
    /// # Returns
    ///
    /// The duration to wait before retrying, or `None` if no more retries.
    fn delay_for(&self, attempt: u32) -> Option<Duration>;

    /// Returns the maximum number of retry attempts allowed.
    ///
    /// # Returns
    ///
    /// The maximum number of attempts, where 0 means no retries.
    fn max_attempts(&self) -> u32;
}
