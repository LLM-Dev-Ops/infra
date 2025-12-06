//! Caching abstraction for LLM-Dev-Ops infrastructure.
//!
//! This crate provides a flexible caching abstraction with support for
//! in-memory and distributed cache implementations.
//!
//! # Examples
//!
//! ```
//! use infra_cache::{Cache, InMemoryCache, CacheConfig};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a cache with default configuration
//! let cache = InMemoryCache::with_defaults();
//!
//! // Store a value
//! cache.set("user:123", "John Doe".to_string(), None).await?;
//!
//! // Retrieve the value
//! let name: Option<String> = cache.get("user:123").await?;
//! assert_eq!(name, Some("John Doe".to_string()));
//!
//! // Store with TTL
//! cache.set("session:456", "active".to_string(), Some(Duration::from_secs(300))).await?;
//!
//! // Delete a value
//! cache.delete("user:123").await?;
//!
//! // Clear the cache
//! cache.clear().await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod cache;
pub mod config;
pub mod error;
pub mod memory;

// Re-export main types
pub use cache::{Cache, CacheEntry};
pub use config::{CacheConfig, EvictionPolicy};
pub use error::{CacheError, CacheResult};
pub use memory::InMemoryCache;
