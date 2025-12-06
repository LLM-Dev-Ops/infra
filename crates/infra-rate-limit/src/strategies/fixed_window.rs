//! Fixed window rate limiting implementation.

use crate::{
    config::RateLimitConfig,
    error::RateLimitError,
    limiter::{RateLimitResult, RateLimiter},
};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::time::{Duration, Instant};

/// Fixed window rate limiter.
///
/// Divides time into fixed windows and allows a maximum number of requests
/// per window. Simple and efficient but can allow bursts at window boundaries.
#[derive(Debug)]
pub struct FixedWindowLimiter {
    config: RateLimitConfig,
    state: Mutex<WindowState>,
}

#[derive(Debug)]
struct WindowState {
    count: u64,
    window_start: Instant,
}

impl FixedWindowLimiter {
    /// Creates a new fixed window rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            state: Mutex::new(WindowState {
                count: 0,
                window_start: Instant::now(),
            }),
        }
    }

    /// Resets the window if it has expired.
    fn maybe_reset_window(&self, state: &mut WindowState) {
        let now = Instant::now();
        let elapsed = now.duration_since(state.window_start);

        if elapsed >= self.config.window_size {
            state.count = 0;
            state.window_start = now;
        }
    }

    /// Calculates wait time until the next window.
    fn calculate_wait_time(&self, state: &WindowState) -> Duration {
        let window_end = state.window_start + self.config.window_size;
        let now = Instant::now();
        window_end.saturating_duration_since(now)
    }
}

#[async_trait]
impl RateLimiter for FixedWindowLimiter {
    async fn acquire(&self) -> Result<(), RateLimitError> {
        loop {
            let result = self.try_acquire().await;
            match result {
                RateLimitResult::Allowed => return Ok(()),
                RateLimitResult::Denied { wait_time } => {
                    tokio::time::sleep(wait_time).await;
                }
            }
        }
    }

    async fn try_acquire(&self) -> RateLimitResult {
        let mut state = self.state.lock();
        self.maybe_reset_window(&mut state);

        if state.count < self.config.burst_size {
            state.count += 1;
            RateLimitResult::Allowed
        } else {
            let wait_time = self.calculate_wait_time(&state);
            RateLimitResult::Denied { wait_time }
        }
    }

    async fn available(&self) -> u64 {
        let mut state = self.state.lock();
        self.maybe_reset_window(&mut state);
        self.config.burst_size.saturating_sub(state.count)
    }

    async fn reset(&self) {
        let mut state = self.state.lock();
        state.count = 0;
        state.window_start = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fixed_window_basic() {
        let config = RateLimitConfig::new(10.0, 5, Duration::from_millis(100)).unwrap();
        let limiter = FixedWindowLimiter::new(config);

        // Should allow up to burst_size
        for _ in 0..5 {
            assert!(limiter.try_acquire().await.is_allowed());
        }

        // Next request should be denied
        assert!(limiter.try_acquire().await.is_denied());
    }

    #[tokio::test]
    async fn test_fixed_window_reset() {
        let config = RateLimitConfig::new(10.0, 5, Duration::from_millis(50)).unwrap();
        let limiter = FixedWindowLimiter::new(config);

        // Fill the window
        for _ in 0..5 {
            limiter.try_acquire().await;
        }

        // Wait for window to reset
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Should allow new requests
        assert!(limiter.try_acquire().await.is_allowed());
    }

    #[tokio::test]
    async fn test_available_count() {
        let config = RateLimitConfig::new(10.0, 5, Duration::from_millis(100)).unwrap();
        let limiter = FixedWindowLimiter::new(config);

        assert_eq!(limiter.available().await, 5);
        limiter.try_acquire().await;
        assert_eq!(limiter.available().await, 4);
    }
}
