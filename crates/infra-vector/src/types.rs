//! Core types for vector operations.
//!
//! This module defines the data types used throughout the infra-vector crate,
//! as specified in SPARC Phase 2 pseudocode.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::fmt;
use std::time::Duration;

/// Unique identifier for a vector.
///
/// Wraps a string ID that uniquely identifies a vector within a collection.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VectorId(String);

impl VectorId {
    /// Create a new VectorId.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a random UUID-based VectorId.
    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VectorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for VectorId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for VectorId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl AsRef<str> for VectorId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Distance metric for similarity calculations.
///
/// From SPARC spec: Corresponds to ruvector_core::Distance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Distance {
    /// Cosine similarity (1 - cosine_distance)
    #[default]
    Cosine,
    /// Euclidean (L2) distance
    Euclidean,
    /// Dot product (inner product)
    DotProduct,
    /// Manhattan (L1) distance
    Manhattan,
}

impl Distance {
    /// Get the string representation for this distance metric.
    pub fn as_str(&self) -> &'static str {
        match self {
            Distance::Cosine => "cosine",
            Distance::Euclidean => "euclidean",
            Distance::DotProduct => "dot_product",
            Distance::Manhattan => "manhattan",
        }
    }
}

impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// HNSW index configuration.
///
/// From SPARC spec: Hierarchical Navigable Small World parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswConfig {
    /// Number of bi-directional links per node (default: 16)
    pub m: usize,
    /// Size of the dynamic candidate list during construction (default: 200)
    pub ef_construction: usize,
    /// Size of the dynamic candidate list during search (default: 100)
    pub ef_search: usize,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ef_search: 100,
        }
    }
}

/// Compression configuration for tiered storage.
///
/// From SPARC spec: Tiered compression (f32→f16→PQ8→PQ4→Binary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Whether compression is enabled
    pub enabled: bool,
    /// Tier thresholds based on access patterns
    pub tier_thresholds: TierThresholds,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tier_thresholds: TierThresholds::default(),
        }
    }
}

/// Tier thresholds for compression decisions.
///
/// Vectors are compressed based on access frequency:
/// - Hot (frequent access): f32 (full precision)
/// - Warm: f16 (half precision)
/// - Cool: PQ8 (product quantization 8-bit)
/// - Cold: PQ4 (product quantization 4-bit)
/// - Archive: Binary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierThresholds {
    /// Access count threshold for hot tier (f32)
    pub hot_access_count: usize,
    /// Access count threshold for warm tier (f16)
    pub warm_access_count: usize,
    /// Access count threshold for cool tier (PQ8)
    pub cool_access_count: usize,
    /// Access count threshold for cold tier (PQ4)
    pub cold_access_count: usize,
    // Below cold -> Binary (archive)
}

impl Default for TierThresholds {
    fn default() -> Self {
        Self {
            hot_access_count: 100,
            warm_access_count: 50,
            cool_access_count: 10,
            cold_access_count: 1,
        }
    }
}

/// Vector store configuration.
///
/// From SPARC spec: Configuration for creating a RuVectorStore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    /// Collection name
    pub collection_name: String,
    /// Vector dimensions
    pub dimensions: usize,
    /// Distance metric
    pub distance: Distance,
    /// HNSW configuration
    pub hnsw: HnswConfig,
    /// Compression configuration
    pub compression: CompressionConfig,
    /// RuvVector endpoint URL (for remote connections)
    pub endpoint_url: Option<String>,
}

impl VectorStoreConfig {
    /// Create a new configuration with required fields.
    pub fn new(collection_name: impl Into<String>, dimensions: usize) -> Self {
        Self {
            collection_name: collection_name.into(),
            dimensions,
            distance: Distance::default(),
            hnsw: HnswConfig::default(),
            compression: CompressionConfig::default(),
            endpoint_url: None,
        }
    }

    /// Set the distance metric.
    pub fn with_distance(mut self, distance: Distance) -> Self {
        self.distance = distance;
        self
    }

    /// Set HNSW configuration.
    pub fn with_hnsw(mut self, hnsw: HnswConfig) -> Self {
        self.hnsw = hnsw;
        self
    }

    /// Set compression configuration.
    pub fn with_compression(mut self, compression: CompressionConfig) -> Self {
        self.compression = compression;
        self
    }

    /// Set the endpoint URL for remote connections.
    pub fn with_endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint_url = Some(url.into());
        self
    }
}

/// A stored vector record.
///
/// From SPARC spec: Complete record including vector data and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRecord {
    /// Unique identifier
    pub id: VectorId,
    /// The vector data
    pub vector: Vec<f32>,
    /// Optional metadata
    pub metadata: Option<Json>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl VectorRecord {
    /// Create a new vector record.
    pub fn new(id: VectorId, vector: Vec<f32>) -> Self {
        let now = Utc::now();
        Self {
            id,
            vector,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set metadata.
    pub fn with_metadata(mut self, metadata: Json) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Search result from a similarity query.
///
/// From SPARC spec: Contains ID, score, and optional vector/metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Vector ID
    pub id: VectorId,
    /// Similarity/distance score
    pub score: f32,
    /// Optional vector data (if requested)
    pub vector: Option<Vec<f32>>,
    /// Optional metadata
    pub metadata: Option<Json>,
}

impl SearchResult {
    /// Create a new search result.
    pub fn new(id: VectorId, score: f32) -> Self {
        Self {
            id,
            score,
            vector: None,
            metadata: None,
        }
    }

    /// Set vector data.
    pub fn with_vector(mut self, vector: Vec<f32>) -> Self {
        self.vector = Some(vector);
        self
    }

    /// Set metadata.
    pub fn with_metadata(mut self, metadata: Json) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Result from a batch insert operation.
///
/// From SPARC spec: Tracks successful and failed inserts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInsertResult {
    /// Number of successfully inserted vectors
    pub inserted: usize,
    /// Failed inserts with their IDs and errors
    pub failed: Vec<(VectorId, String)>,
    /// Duration of the batch operation
    #[serde(with = "duration_serde")]
    pub duration: Duration,
}

impl BatchInsertResult {
    /// Create a new batch result.
    pub fn new(inserted: usize, failed: Vec<(VectorId, String)>, duration: Duration) -> Self {
        Self {
            inserted,
            failed,
            duration,
        }
    }

    /// Check if all inserts succeeded.
    pub fn all_succeeded(&self) -> bool {
        self.failed.is_empty()
    }

    /// Get total attempted inserts.
    pub fn total(&self) -> usize {
        self.inserted + self.failed.len()
    }

    /// Get success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            100.0
        } else {
            (self.inserted as f64 / total as f64) * 100.0
        }
    }
}

/// Collection statistics.
///
/// From SPARC spec: Information about a vector collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    /// Total number of vectors
    pub total_vectors: usize,
    /// Vector dimensions
    pub dimensions: usize,
    /// Size of the index in bytes
    pub index_size_bytes: usize,
    /// Distance metric used
    pub distance_metric: Distance,
    /// Collection name
    pub collection_name: String,
}

impl CollectionStats {
    /// Create new collection stats.
    pub fn new(
        collection_name: impl Into<String>,
        total_vectors: usize,
        dimensions: usize,
        distance_metric: Distance,
    ) -> Self {
        Self {
            total_vectors,
            dimensions,
            index_size_bytes: 0,
            distance_metric,
            collection_name: collection_name.into(),
        }
    }
}

/// Metadata filter for search queries.
///
/// From SPARC spec: Supports various filter operations that convert to ruvector's filter format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum MetadataFilter {
    /// Equals comparison
    Eq { field: String, value: Json },
    /// Not equals comparison
    Ne { field: String, value: Json },
    /// Greater than comparison
    Gt { field: String, value: Json },
    /// Greater than or equal comparison
    Gte { field: String, value: Json },
    /// Less than comparison
    Lt { field: String, value: Json },
    /// Less than or equal comparison
    Lte { field: String, value: Json },
    /// In set comparison
    In { field: String, values: Vec<Json> },
    /// String contains
    Contains { field: String, value: String },
    /// Logical AND of multiple filters
    And(Vec<MetadataFilter>),
    /// Logical OR of multiple filters
    Or(Vec<MetadataFilter>),
    /// Logical NOT
    Not(Box<MetadataFilter>),
}

impl MetadataFilter {
    /// Create an equality filter.
    pub fn eq(field: impl Into<String>, value: impl Into<Json>) -> Self {
        Self::Eq {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a not-equals filter.
    pub fn ne(field: impl Into<String>, value: impl Into<Json>) -> Self {
        Self::Ne {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a greater-than filter.
    pub fn gt(field: impl Into<String>, value: impl Into<Json>) -> Self {
        Self::Gt {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a greater-than-or-equal filter.
    pub fn gte(field: impl Into<String>, value: impl Into<Json>) -> Self {
        Self::Gte {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a less-than filter.
    pub fn lt(field: impl Into<String>, value: impl Into<Json>) -> Self {
        Self::Lt {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a less-than-or-equal filter.
    pub fn lte(field: impl Into<String>, value: impl Into<Json>) -> Self {
        Self::Lte {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create an IN filter.
    pub fn in_set(field: impl Into<String>, values: Vec<Json>) -> Self {
        Self::In {
            field: field.into(),
            values,
        }
    }

    /// Create a contains filter.
    pub fn contains(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Contains {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create an AND filter.
    pub fn and(filters: Vec<MetadataFilter>) -> Self {
        Self::And(filters)
    }

    /// Create an OR filter.
    pub fn or(filters: Vec<MetadataFilter>) -> Self {
        Self::Or(filters)
    }

    /// Create a NOT filter.
    pub fn not(filter: MetadataFilter) -> Self {
        Self::Not(Box::new(filter))
    }
}

// Helper module for serializing Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_id() {
        let id = VectorId::new("test-123");
        assert_eq!(id.as_str(), "test-123");
        assert_eq!(id.to_string(), "test-123");
    }

    #[test]
    fn test_distance_default() {
        let d = Distance::default();
        assert_eq!(d, Distance::Cosine);
    }

    #[test]
    fn test_hnsw_config_default() {
        let config = HnswConfig::default();
        assert_eq!(config.m, 16);
        assert_eq!(config.ef_construction, 200);
        assert_eq!(config.ef_search, 100);
    }

    #[test]
    fn test_vector_store_config() {
        let config = VectorStoreConfig::new("test_collection", 1536)
            .with_distance(Distance::Euclidean)
            .with_endpoint("http://localhost:8100");

        assert_eq!(config.collection_name, "test_collection");
        assert_eq!(config.dimensions, 1536);
        assert_eq!(config.distance, Distance::Euclidean);
        assert_eq!(config.endpoint_url, Some("http://localhost:8100".to_string()));
    }

    #[test]
    fn test_metadata_filter() {
        let filter = MetadataFilter::and(vec![
            MetadataFilter::eq("category", "tech"),
            MetadataFilter::gte("score", 0.8),
        ]);

        match filter {
            MetadataFilter::And(filters) => {
                assert_eq!(filters.len(), 2);
            }
            _ => panic!("Expected And filter"),
        }
    }

    #[test]
    fn test_batch_result() {
        let result = BatchInsertResult::new(
            90,
            vec![(VectorId::new("failed-1"), "error".to_string())],
            Duration::from_millis(100),
        );

        assert_eq!(result.total(), 91);
        assert!(!result.all_succeeded());
        assert!((result.success_rate() - 98.9).abs() < 0.1);
    }
}
