//! Core cache trait and types.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::time::{Duration, SystemTime};

use crate::error::CacheResult;

/// A cache entry with optional TTL.
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// The cached value.
    pub value: T,
    /// When the entry was created.
    pub created_at: SystemTime,
    /// Time-to-live for this entry.
    pub ttl: Option<Duration>,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry with no expiration.
    pub fn new(value: T) -> Self {
        Self {
            value,
            created_at: SystemTime::now(),
            ttl: None,
        }
    }

    /// Create a new cache entry with the specified TTL.
    pub fn with_ttl(value: T, ttl: Duration) -> Self {
        Self {
            value,
            created_at: SystemTime::now(),
            ttl: Some(ttl),
        }
    }

    /// Check if this entry has expired.
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            if let Ok(elapsed) = self.created_at.elapsed() {
                return elapsed > ttl;
            }
        }
        false
    }

    /// Get the remaining time until expiration.
    pub fn time_to_expiry(&self) -> Option<Duration> {
        self.ttl.and_then(|ttl| {
            self.created_at
                .elapsed()
                .ok()
                .and_then(|elapsed| ttl.checked_sub(elapsed))
        })
    }
}

/// Async cache trait for storing and retrieving data.
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get a value from the cache.
    ///
    /// Returns `None` if the key doesn't exist or the entry has expired.
    async fn get<T>(&self, key: &str) -> CacheResult<Option<T>>
    where
        T: DeserializeOwned + Send + 'static;

    /// Set a value in the cache with optional TTL.
    ///
    /// If TTL is `None`, uses the cache's default TTL (if configured).
    async fn set<T>(&self, key: &str, value: T, ttl: Option<Duration>) -> CacheResult<()>
    where
        T: Serialize + Send + Sync + 'static;

    /// Delete a value from the cache.
    ///
    /// Returns `true` if the key existed, `false` otherwise.
    async fn delete(&self, key: &str) -> CacheResult<bool>;

    /// Clear all entries from the cache.
    async fn clear(&self) -> CacheResult<()>;

    /// Check if a key exists in the cache.
    ///
    /// Returns `false` if the key doesn't exist or has expired.
    async fn exists(&self, key: &str) -> CacheResult<bool>;

    /// Get the number of entries in the cache.
    async fn len(&self) -> CacheResult<usize>;

    /// Check if the cache is empty.
    async fn is_empty(&self) -> CacheResult<bool> {
        Ok(self.len().await? == 0)
    }
}
