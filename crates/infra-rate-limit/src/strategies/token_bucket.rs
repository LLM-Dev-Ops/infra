//! Token bucket rate limiting implementation.

use crate::{
    config::RateLimitConfig,
    error::RateLimitError,
    limiter::{RateLimitResult, RateLimiter},
};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::time::{Duration, Instant};

/// Token bucket rate limiter.
///
/// The token bucket algorithm allows bursts while maintaining a steady rate.
/// Tokens are added at a constant rate, and each request consumes a token.
#[derive(Debug)]
pub struct TokenBucket {
    config: RateLimitConfig,
    state: Mutex<BucketState>,
}

#[derive(Debug)]
struct BucketState {
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucket {
    /// Creates a new token bucket rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            state: Mutex::new(BucketState {
                tokens: config.burst_size as f64,
                last_refill: Instant::now(),
            }),
        }
    }

    /// Refills tokens based on elapsed time.
    fn refill(&self, state: &mut BucketState) {
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill);
        let new_tokens = elapsed.as_secs_f64() * self.config.requests_per_second;

        state.tokens = (state.tokens + new_tokens).min(self.config.burst_size as f64);
        state.last_refill = now;
    }

    /// Calculates wait time for next token.
    fn calculate_wait_time(&self, tokens_needed: f64) -> Duration {
        let time_per_token = 1.0 / self.config.requests_per_second;
        let wait_seconds = time_per_token * tokens_needed;
        Duration::from_secs_f64(wait_seconds)
    }
}

#[async_trait]
impl RateLimiter for TokenBucket {
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
        self.refill(&mut state);

        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            RateLimitResult::Allowed
        } else {
            let tokens_needed = 1.0 - state.tokens;
            let wait_time = self.calculate_wait_time(tokens_needed);
            RateLimitResult::Denied { wait_time }
        }
    }

    async fn available(&self) -> u64 {
        let mut state = self.state.lock();
        self.refill(&mut state);
        state.tokens.floor() as u64
    }

    async fn reset(&self) {
        let mut state = self.state.lock();
        state.tokens = self.config.burst_size as f64;
        state.last_refill = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_bucket_allows_burst() {
        let config = RateLimitConfig::per_second(10.0).unwrap();
        let limiter = TokenBucket::new(config);

        // Should allow burst up to burst_size
        for _ in 0..10 {
            assert!(limiter.try_acquire().await.is_allowed());
        }

        // Next request should be denied
        assert!(limiter.try_acquire().await.is_denied());
    }

    #[tokio::test]
    async fn test_token_bucket_refill() {
        let config = RateLimitConfig::per_second(10.0).unwrap();
        let limiter = TokenBucket::new(config);

        // Consume all tokens
        for _ in 0..10 {
            limiter.try_acquire().await;
        }

        // Wait for refill
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should have at least 1 token available
        assert!(limiter.available().await >= 1);
    }
}
