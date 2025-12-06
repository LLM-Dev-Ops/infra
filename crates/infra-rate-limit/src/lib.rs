//! Rate limiting utilities for LLM-Dev-Ops infrastructure.
//!
//! This crate provides various rate limiting strategies for controlling
//! request rates in distributed systems.
//!
//! # Features
//!
//! - Multiple rate limiting algorithms (token bucket, sliding window, fixed window)
//! - Async/await support via tokio
//! - Thread-safe implementations
//! - Configurable rate limits and burst sizes
//!
//! # Examples
//!
//! ## Token Bucket
//!
//! ```rust
//! use infra_rate_limit::{RateLimitConfig, TokenBucket, RateLimiter};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RateLimitConfig::per_second(100.0)?;
//! let limiter = TokenBucket::new(config);
//!
//! // Acquire a permit
//! limiter.acquire().await?;
//!
//! // Try to acquire without waiting
//! let result = limiter.try_acquire().await;
//! if result.is_allowed() {
//!     // Process request
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Sliding Window
//!
//! ```rust
//! use infra_rate_limit::{RateLimitConfig, SlidingWindowLimiter, RateLimiter};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RateLimitConfig::new(10.0, 10, Duration::from_secs(1))?;
//! let limiter = SlidingWindowLimiter::new(config);
//!
//! limiter.acquire().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Fixed Window
//!
//! ```rust
//! use infra_rate_limit::{RateLimitConfig, FixedWindowLimiter, RateLimiter};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RateLimitConfig::per_minute(1000.0)?;
//! let limiter = FixedWindowLimiter::new(config);
//!
//! // Check available permits
//! let available = limiter.available().await;
//! println!("Available permits: {}", available);
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod config;
pub mod error;
pub mod limiter;
pub mod strategies;

// Re-exports
pub use config::RateLimitConfig;
pub use error::RateLimitError;
pub use limiter::{RateLimitResult, RateLimiter};
pub use strategies::{FixedWindowLimiter, SlidingWindowLimiter, TokenBucket};
