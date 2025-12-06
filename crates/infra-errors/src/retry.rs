//! Retry configuration and strategies.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for retry behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,

    /// Base delay between retries
    pub base_delay: Duration,

    /// Maximum delay between retries
    pub max_delay: Duration,

    /// Whether to add jitter to delays
    pub jitter: bool,

    /// Retry strategy to use
    pub strategy: RetryStrategy,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            jitter: true,
            strategy: RetryStrategy::ExponentialBackoff,
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with default settings
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum attempts
    #[must_use]
    pub fn with_max_attempts(mut self, attempts: usize) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Set base delay
    #[must_use]
    pub fn with_base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Set maximum delay
    #[must_use]
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Enable or disable jitter
    #[must_use]
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Set retry strategy
    #[must_use]
    pub fn with_strategy(mut self, strategy: RetryStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Calculate delay for a given attempt number (0-indexed)
    #[must_use]
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        let base_ms = self.base_delay.as_millis() as u64;
        let max_ms = self.max_delay.as_millis() as u64;

        let delay_ms = match self.strategy {
            RetryStrategy::Constant => base_ms,
            RetryStrategy::Linear => base_ms * (attempt as u64 + 1),
            RetryStrategy::ExponentialBackoff => {
                base_ms.saturating_mul(2u64.saturating_pow(attempt as u32))
            }
        };

        let capped_ms = delay_ms.min(max_ms);

        #[cfg(feature = "rand")]
        let final_ms = if self.jitter {
            // Add up to 25% jitter
            let jitter_range = capped_ms / 4;
            if jitter_range > 0 {
                capped_ms + (rand::random::<u64>() % jitter_range)
            } else {
                capped_ms
            }
        } else {
            capped_ms
        };

        #[cfg(not(feature = "rand"))]
        let final_ms = capped_ms;

        Duration::from_millis(final_ms)
    }

    /// Create a config optimized for fast retries
    #[must_use]
    pub fn fast() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            jitter: false,
            strategy: RetryStrategy::Constant,
        }
    }

    /// Create a config optimized for slow/rate-limited services
    #[must_use]
    pub fn slow() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            jitter: true,
            strategy: RetryStrategy::ExponentialBackoff,
        }
    }
}

/// Strategy for calculating retry delays
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// Use constant delay between retries
    Constant,

    /// Linearly increase delay
    Linear,

    /// Exponentially increase delay (2^attempt * base)
    ExponentialBackoff,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let config = RetryConfig::new()
            .with_base_delay(Duration::from_millis(100))
            .with_jitter(false);

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(400));
        assert_eq!(config.calculate_delay(3), Duration::from_millis(800));
    }

    #[test]
    fn test_max_delay_cap() {
        let config = RetryConfig::new()
            .with_base_delay(Duration::from_secs(1))
            .with_max_delay(Duration::from_secs(5))
            .with_jitter(false);

        // Should be capped at 5 seconds
        assert_eq!(config.calculate_delay(10), Duration::from_secs(5));
    }

    #[test]
    fn test_constant_strategy() {
        let config = RetryConfig::new()
            .with_strategy(RetryStrategy::Constant)
            .with_base_delay(Duration::from_millis(50))
            .with_jitter(false);

        assert_eq!(config.calculate_delay(0), Duration::from_millis(50));
        assert_eq!(config.calculate_delay(5), Duration::from_millis(50));
    }
}
