//! Cache configuration types.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cache eviction policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used - evicts the least recently accessed item.
    LRU,
    /// Least Frequently Used - evicts the least frequently accessed item.
    LFU,
    /// First In First Out - evicts the oldest item.
    FIFO,
}

impl Default for EvictionPolicy {
    fn default() -> Self {
        Self::LRU
    }
}

/// Cache configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache.
    /// None means unlimited.
    pub max_size: Option<usize>,

    /// Default time-to-live for cache entries.
    /// None means entries don't expire by default.
    #[serde(
        serialize_with = "serialize_duration_option",
        deserialize_with = "deserialize_duration_option"
    )]
    pub default_ttl: Option<Duration>,

    /// Eviction policy to use when cache is full.
    pub eviction_policy: EvictionPolicy,

    /// Enable metrics collection.
    #[serde(default)]
    pub enable_metrics: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: Some(1000),
            default_ttl: Some(Duration::from_secs(3600)), // 1 hour
            eviction_policy: EvictionPolicy::LRU,
            enable_metrics: false,
        }
    }
}

impl CacheConfig {
    /// Create a new cache configuration with unlimited size.
    pub fn unlimited() -> Self {
        Self {
            max_size: None,
            default_ttl: None,
            eviction_policy: EvictionPolicy::LRU,
            enable_metrics: false,
        }
    }

    /// Create a new cache configuration with the specified max size.
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            max_size: Some(max_size),
            ..Default::default()
        }
    }

    /// Set the default TTL.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = Some(ttl);
        self
    }

    /// Set the eviction policy.
    pub fn with_eviction_policy(mut self, policy: EvictionPolicy) -> Self {
        self.eviction_policy = policy;
        self
    }

    /// Enable metrics collection.
    pub fn with_metrics(mut self, enable: bool) -> Self {
        self.enable_metrics = enable;
        self
    }
}

// Serde helpers for Duration
fn serialize_duration_option<S>(
    duration: &Option<Duration>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match duration {
        Some(d) => serializer.serialize_some(&d.as_secs()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_duration_option<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs: Option<u64> = Option::deserialize(deserializer)?;
    Ok(secs.map(Duration::from_secs))
}
