//! Advanced retry policies for LLM-Dev-Ops infrastructure.
//!
//! This crate provides flexible retry mechanisms with various built-in strategies
//! including exponential backoff, fixed delays, and jitter support.
//!
//! # Features
//!
//! - `std` (default): Enables standard library support.
//!
//! # Examples
//!
//! ```no_run
//! use infra_retry::{retry_with_policy, ExponentialBackoff};
//! use std::io;
//!
//! # async fn example() -> Result<(), io::Error> {
//! let policy = ExponentialBackoff::default()
//!     .with_max_attempts(5)
//!     .with_initial_delay(std::time::Duration::from_millis(100));
//!
//! let mut count = 0;
//! let result = retry_with_policy(
//!     || async {
//!         count += 1;
//!         if count < 3 {
//!             Err(io::Error::new(io::ErrorKind::Other, "temporary"))
//!         } else {
//!             Ok("success")
//!         }
//!     },
//!     &policy,
//! ).await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod executor;
pub mod policy;
pub mod strategies;

// Re-export key types for convenience
pub use executor::{retry_retryable, retry_with_policy, Retryable};
pub use policy::{RetryDecision, RetryPolicy};
pub use strategies::{ExponentialBackoff, FixedDelay, WithJitter};
