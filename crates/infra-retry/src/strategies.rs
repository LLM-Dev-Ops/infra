//! Built-in retry strategy implementations.

use crate::policy::{RetryDecision, RetryPolicy};
use rand::Rng;
use std::time::Duration;

/// Exponential backoff retry strategy.
///
/// Each retry attempt waits exponentially longer than the previous one.
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    /// Initial delay before the first retry.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Multiplier for each subsequent retry.
    pub multiplier: f64,
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
}

impl ExponentialBackoff {
    /// Creates a new exponential backoff policy with default values.
    pub fn new() -> Self {
        Self {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            max_attempts: 5,
        }
    }

    /// Sets the initial delay.
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Sets the maximum delay.
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Sets the multiplier.
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Sets the maximum number of attempts.
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new()
    }
}

impl RetryPolicy for ExponentialBackoff {
    fn should_retry(&self, attempt: u32, _error: &dyn std::error::Error) -> RetryDecision {
        if attempt >= self.max_attempts {
            return RetryDecision::Stop;
        }

        if let Some(delay) = self.delay_for(attempt) {
            RetryDecision::Retry(delay)
        } else {
            RetryDecision::Stop
        }
    }

    fn delay_for(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None;
        }

        let delay_ms = self.initial_delay.as_millis() as f64
            * self.multiplier.powi(attempt as i32);
        let delay = Duration::from_millis(delay_ms as u64);

        Some(delay.min(self.max_delay))
    }

    fn max_attempts(&self) -> u32 {
        self.max_attempts
    }
}

/// Fixed delay retry strategy.
///
/// Each retry attempt waits for a constant duration.
#[derive(Debug, Clone)]
pub struct FixedDelay {
    /// The fixed delay between retries.
    pub delay: Duration,
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
}

impl FixedDelay {
    /// Creates a new fixed delay policy.
    pub fn new(delay: Duration, max_attempts: u32) -> Self {
        Self {
            delay,
            max_attempts,
        }
    }
}

impl RetryPolicy for FixedDelay {
    fn should_retry(&self, attempt: u32, _error: &dyn std::error::Error) -> RetryDecision {
        if attempt >= self.max_attempts {
            return RetryDecision::Stop;
        }

        RetryDecision::Retry(self.delay)
    }

    fn delay_for(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            None
        } else {
            Some(self.delay)
        }
    }

    fn max_attempts(&self) -> u32 {
        self.max_attempts
    }
}

/// Wrapper that adds jitter to another retry policy.
///
/// Jitter helps prevent thundering herd problems by randomizing delays.
#[derive(Debug, Clone)]
pub struct WithJitter<P> {
    /// The underlying retry policy.
    pub inner: P,
    /// Maximum jitter as a fraction of the delay (0.0 to 1.0).
    pub jitter_factor: f64,
}

impl<P> WithJitter<P> {
    /// Creates a new jittered policy wrapping another policy.
    pub fn new(inner: P, jitter_factor: f64) -> Self {
        Self {
            inner,
            jitter_factor: jitter_factor.clamp(0.0, 1.0),
        }
    }

    /// Applies jitter to a duration.
    fn apply_jitter(&self, duration: Duration) -> Duration {
        if self.jitter_factor == 0.0 {
            return duration;
        }

        let mut rng = rand::thread_rng();
        let jitter_range = duration.as_millis() as f64 * self.jitter_factor;
        let jitter = rng.gen_range(0.0..=jitter_range);

        let base_delay = duration.as_millis() as f64;
        let jittered_delay = base_delay - (jitter_range / 2.0) + jitter;

        Duration::from_millis(jittered_delay.max(0.0) as u64)
    }
}

impl<P: RetryPolicy> RetryPolicy for WithJitter<P> {
    fn should_retry(&self, attempt: u32, error: &dyn std::error::Error) -> RetryDecision {
        match self.inner.should_retry(attempt, error) {
            RetryDecision::Retry(delay) => {
                RetryDecision::Retry(self.apply_jitter(delay))
            }
            RetryDecision::Stop => RetryDecision::Stop,
        }
    }

    fn delay_for(&self, attempt: u32) -> Option<Duration> {
        self.inner.delay_for(attempt).map(|d| self.apply_jitter(d))
    }

    fn max_attempts(&self) -> u32 {
        self.inner.max_attempts()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff_default() {
        let policy = ExponentialBackoff::default();
        assert_eq!(policy.max_attempts(), 5);

        let delay0 = policy.delay_for(0).unwrap();
        let delay1 = policy.delay_for(1).unwrap();
        assert!(delay1 > delay0);
    }

    #[test]
    fn test_fixed_delay() {
        let policy = FixedDelay::new(Duration::from_secs(1), 3);
        assert_eq!(policy.max_attempts(), 3);

        let delay0 = policy.delay_for(0).unwrap();
        let delay1 = policy.delay_for(1).unwrap();
        assert_eq!(delay0, delay1);
    }

    #[test]
    fn test_with_jitter() {
        let base = FixedDelay::new(Duration::from_secs(1), 3);
        let jittered = WithJitter::new(base, 0.5);

        assert_eq!(jittered.max_attempts(), 3);

        // Jittered delays should be different from base (probabilistically)
        let delay = jittered.delay_for(0).unwrap();
        assert!(delay.as_millis() > 0);
    }
}
