//! In-memory cache implementation.

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::cache::{Cache, CacheEntry};
use crate::config::CacheConfig;
use crate::error::{CacheError, CacheResult};

/// Internal cache entry that stores serialized data.
#[derive(Debug, Clone)]
struct InternalEntry {
    data: Vec<u8>,
    entry: CacheEntry<()>,
}

/// In-memory cache implementation using DashMap.
#[derive(Debug, Clone)]
pub struct InMemoryCache {
    store: Arc<DashMap<String, InternalEntry>>,
    config: Arc<CacheConfig>,
}

impl InMemoryCache {
    /// Create a new in-memory cache with the given configuration.
    pub fn new(config: CacheConfig) -> Self {
        Self {
            store: Arc::new(DashMap::new()),
            config: Arc::new(config),
        }
    }

    /// Create a new in-memory cache with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Create a new in-memory cache with unlimited size.
    pub fn unlimited() -> Self {
        Self::new(CacheConfig::unlimited())
    }

    /// Remove expired entries from the cache.
    fn evict_expired(&self) {
        self.store.retain(|_, entry| !entry.entry.is_expired());
    }

    /// Check if the cache is full and needs eviction.
    fn needs_eviction(&self) -> bool {
        if let Some(max_size) = self.config.max_size {
            self.store.len() >= max_size
        } else {
            false
        }
    }

    /// Evict one entry according to the eviction policy.
    fn evict_one(&self) -> CacheResult<()> {
        // First, try to remove expired entries
        self.evict_expired();

        // If still full, remove based on eviction policy
        if self.needs_eviction() {
            // For now, just remove the first entry (FIFO-like behavior)
            // TODO: Implement proper LRU/LFU tracking
            if let Some(entry) = self.store.iter().next() {
                let key = entry.key().clone();
                drop(entry);
                self.store.remove(&key);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn get<T>(&self, key: &str) -> CacheResult<Option<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        // Remove expired entries periodically
        if self.store.len() % 100 == 0 {
            self.evict_expired();
        }

        if let Some(entry) = self.store.get(key) {
            // Check if expired
            if entry.entry.is_expired() {
                drop(entry);
                self.store.remove(key);
                return Ok(None);
            }

            // Deserialize the value
            let value: T = serde_json::from_slice(&entry.data).map_err(|e| {
                CacheError::DeserializationError(format!("Failed to deserialize: {}", e))
            })?;

            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    async fn set<T>(&self, key: &str, value: T, ttl: Option<Duration>) -> CacheResult<()>
    where
        T: Serialize + Send + Sync + 'static,
    {
        // Check if we need to evict entries
        if self.needs_eviction() && !self.store.contains_key(key) {
            self.evict_one()?;
        }

        // Serialize the value
        let data = serde_json::to_vec(&value)?;

        // Determine TTL
        let entry_ttl = ttl.or(self.config.default_ttl);

        // Create the entry
        let entry = if let Some(ttl) = entry_ttl {
            CacheEntry::with_ttl((), ttl)
        } else {
            CacheEntry::new(())
        };

        let internal_entry = InternalEntry { data, entry };

        // Store the entry
        self.store.insert(key.to_string(), internal_entry);

        Ok(())
    }

    async fn delete(&self, key: &str) -> CacheResult<bool> {
        Ok(self.store.remove(key).is_some())
    }

    async fn clear(&self) -> CacheResult<()> {
        self.store.clear();
        Ok(())
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        if let Some(entry) = self.store.get(key) {
            if entry.entry.is_expired() {
                drop(entry);
                self.store.remove(key);
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    async fn len(&self) -> CacheResult<usize> {
        // Remove expired entries before counting
        self.evict_expired();
        Ok(self.store.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_basic_operations() {
        let cache = InMemoryCache::with_defaults();

        // Test set and get
        cache
            .set("key1", "value1".to_string(), None)
            .await
            .unwrap();
        let result: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(result, Some("value1".to_string()));

        // Test exists
        assert!(cache.exists("key1").await.unwrap());
        assert!(!cache.exists("key2").await.unwrap());

        // Test delete
        assert!(cache.delete("key1").await.unwrap());
        assert!(!cache.delete("key1").await.unwrap());

        // Test len and is_empty
        cache
            .set("key2", "value2".to_string(), None)
            .await
            .unwrap();
        assert_eq!(cache.len().await.unwrap(), 1);
        assert!(!cache.is_empty().await.unwrap());

        // Test clear
        cache.clear().await.unwrap();
        assert!(cache.is_empty().await.unwrap());
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = InMemoryCache::with_defaults();

        // Set with very short TTL
        cache
            .set("key1", "value1".to_string(), Some(Duration::from_millis(50)))
            .await
            .unwrap();

        // Should exist immediately
        assert!(cache.exists("key1").await.unwrap());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should not exist after expiration
        assert!(!cache.exists("key1").await.unwrap());
        let result: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_max_size() {
        let config = CacheConfig::with_max_size(2);
        let cache = InMemoryCache::new(config);

        // Fill the cache
        cache
            .set("key1", "value1".to_string(), None)
            .await
            .unwrap();
        cache
            .set("key2", "value2".to_string(), None)
            .await
            .unwrap();

        // Adding a third item should evict one
        cache
            .set("key3", "value3".to_string(), None)
            .await
            .unwrap();

        // Cache should have at most 2 items
        assert!(cache.len().await.unwrap() <= 2);
    }
}
