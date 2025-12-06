//! Configuration types for rate limiters.

use crate::error::RateLimitError;
use std::time::Duration;

/// Configuration for rate limiting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RateLimitConfig {
    /// Maximum number of requests per second.
    pub requests_per_second: f64,

    /// Maximum burst size (tokens available at once).
    pub burst_size: u64,

    /// Window size for windowed rate limiters.
    pub window_size: Duration,
}

impl RateLimitConfig {
    /// Creates a new rate limit configuration.
    ///
    /// # Arguments
    ///
    /// * `requests_per_second` - Maximum requests per second
    /// * `burst_size` - Maximum burst size
    /// * `window_size` - Window size for windowed limiters
    pub fn new(
        requests_per_second: f64,
        burst_size: u64,
        window_size: Duration,
    ) -> Result<Self, RateLimitError> {
        if requests_per_second <= 0.0 {
            return Err(RateLimitError::invalid_config(
                "requests_per_second must be positive",
            ));
        }
        if burst_size == 0 {
            return Err(RateLimitError::invalid_config("burst_size must be positive"));
        }

        Ok(Self {
            requests_per_second,
            burst_size,
            window_size,
        })
    }

    /// Creates a configuration with requests per second and default burst.
    pub fn per_second(requests_per_second: f64) -> Result<Self, RateLimitError> {
        Self::new(
            requests_per_second,
            requests_per_second.ceil() as u64,
            Duration::from_secs(1),
        )
    }

    /// Creates a configuration with requests per minute.
    pub fn per_minute(requests_per_minute: f64) -> Result<Self, RateLimitError> {
        Self::new(
            requests_per_minute / 60.0,
            requests_per_minute.ceil() as u64,
            Duration::from_secs(60),
        )
    }

    /// Creates a configuration with requests per hour.
    pub fn per_hour(requests_per_hour: f64) -> Result<Self, RateLimitError> {
        Self::new(
            requests_per_hour / 3600.0,
            requests_per_hour.ceil() as u64,
            Duration::from_secs(3600),
        )
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10.0,
            burst_size: 10,
            window_size: Duration::from_secs(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        assert!(RateLimitConfig::new(10.0, 10, Duration::from_secs(1)).is_ok());
        assert!(RateLimitConfig::new(0.0, 10, Duration::from_secs(1)).is_err());
        assert!(RateLimitConfig::new(10.0, 0, Duration::from_secs(1)).is_err());
    }

    #[test]
    fn test_per_second() {
        let config = RateLimitConfig::per_second(100.0).unwrap();
        assert_eq!(config.requests_per_second, 100.0);
        assert_eq!(config.burst_size, 100);
    }
}
