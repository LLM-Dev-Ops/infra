//! Sliding window rate limiting implementation.

use crate::{
    config::RateLimitConfig,
    error::RateLimitError,
    limiter::{RateLimitResult, RateLimiter},
};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// Sliding window rate limiter.
///
/// Tracks requests in a sliding time window. More accurate than fixed window
/// but requires more memory to track individual request timestamps.
#[derive(Debug)]
pub struct SlidingWindowLimiter {
    config: RateLimitConfig,
    state: Mutex<WindowState>,
}

#[derive(Debug)]
struct WindowState {
    requests: VecDeque<Instant>,
}

impl SlidingWindowLimiter {
    /// Creates a new sliding window rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            state: Mutex::new(WindowState {
                requests: VecDeque::new(),
            }),
        }
    }

    /// Removes expired requests from the window.
    fn clean_expired(&self, state: &mut WindowState, now: Instant) {
        let cutoff = now - self.config.window_size;
        while let Some(&first) = state.requests.front() {
            if first < cutoff {
                state.requests.pop_front();
            } else {
                break;
            }
        }
    }

    /// Calculates wait time until the next slot becomes available.
    fn calculate_wait_time(&self, state: &WindowState, now: Instant) -> Duration {
        if let Some(&oldest) = state.requests.front() {
            let window_end = oldest + self.config.window_size;
            window_end.saturating_duration_since(now)
        } else {
            Duration::ZERO
        }
    }
}

#[async_trait]
impl RateLimiter for SlidingWindowLimiter {
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
        let now = Instant::now();

        self.clean_expired(&mut state, now);

        if (state.requests.len() as u64) < self.config.burst_size {
            state.requests.push_back(now);
            RateLimitResult::Allowed
        } else {
            let wait_time = self.calculate_wait_time(&state, now);
            RateLimitResult::Denied { wait_time }
        }
    }

    async fn available(&self) -> u64 {
        let mut state = self.state.lock();
        let now = Instant::now();

        self.clean_expired(&mut state, now);

        self.config.burst_size.saturating_sub(state.requests.len() as u64)
    }

    async fn reset(&self) {
        let mut state = self.state.lock();
        state.requests.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sliding_window_basic() {
        let config = RateLimitConfig::new(10.0, 5, Duration::from_millis(100)).unwrap();
        let limiter = SlidingWindowLimiter::new(config);

        // Should allow up to burst_size
        for _ in 0..5 {
            assert!(limiter.try_acquire().await.is_allowed());
        }

        // Next request should be denied
        assert!(limiter.try_acquire().await.is_denied());
    }

    #[tokio::test]
    async fn test_sliding_window_cleanup() {
        let config = RateLimitConfig::new(10.0, 5, Duration::from_millis(50)).unwrap();
        let limiter = SlidingWindowLimiter::new(config);

        // Fill the window
        for _ in 0..5 {
            limiter.try_acquire().await;
        }

        // Wait for window to slide
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Should allow new requests
        assert!(limiter.try_acquire().await.is_allowed());
    }
}
